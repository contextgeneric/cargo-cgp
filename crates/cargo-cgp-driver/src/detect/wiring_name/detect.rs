//! Recognizing a wiring key, a wiring value, or a `#[cgp_impl]` provider that does not resolve.
//!
//! A consumer trait written as a key fails while the compiler lowers the generated impl's header
//! (`E0782`), so it is recognized as it is emitted ([`detect_consumer_trait_as_key`]). A name that
//! resolves to nothing fails during name resolution (`E0425`), before the HIR exists, so those are
//! recognized together once resolution is over ([`plan_unresolved_names`]), from the driver's
//! `after_expansion` callback.

use cargo_cgp_error_processing::{UnresolvedWiringName, similar_name};
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{QPath, TyKind};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::detect::wiring_name::component::{
    component_markers, consumer_component, consumer_trait_named, is_type_declared,
};
use crate::detect::wiring_name::slot::{NameSlot, name_slot_at};
use crate::resolve::cgp_item::{is_consumer_trait, marker_to_consumer};
use crate::resolve::query::import_path;

/// What to do with the name-resolution diagnostics of the wiring entries, and the overflows to
/// expect from them.
pub struct UnresolvedNamesPlan {
    /// The action for each diagnostic, in the order given.
    pub actions: Vec<Option<NameAction>>,
    /// The requirements (`Person: CanGreet`) a wiring entry whose value does not resolve leaves
    /// unprovable, as the compiler prints them: a later `E0275` overflow on one is that entry's
    /// consequence.
    pub overflow_requirements: Vec<String>,
}

/// What to do with a diagnostic about a name that does not resolve.
pub enum NameAction {
    /// Reword it into the coded form of this mistake.
    Reword(UnresolvedWiringName),
    /// Reword only its header into the coded form of this mistake, keeping the compiler's own notes
    /// and suggestions, which are right for the mistake except for a type parameter to add.
    RewordHeader(UnresolvedWiringName),
    /// Drop it: a repeat of a diagnostic at the same span, or the consequence of another's mistake.
    Drop,
}

/// Recognize an `E0782` at `span` as a consumer trait written as a wiring key: the span is the key
/// of a wiring entry, the key resolves to a CGP consumer trait, and that trait has a component.
pub fn detect_consumer_trait_as_key(tcx: TyCtxt<'_>, span: Span) -> Option<UnresolvedWiringName> {
    let NameSlot::Key { key } = name_slot_at(tcx, span)? else {
        return None;
    };
    // A bare trait in a type position lowers to a trait object, or to a path in an edition that
    // still accepts one.
    let res = match key.kind {
        TyKind::TraitObject([bound], _) => bound.trait_ref.path.res,
        TyKind::Path(QPath::Resolved(None, path)) => path.res,
        _ => return None,
    };
    let Res::Def(DefKind::Trait, consumer_did) = res else {
        return None;
    };
    if !is_consumer_trait(tcx, consumer_did) {
        return None;
    }
    let component_did = consumer_component(tcx, consumer_did)?;
    Some(UnresolvedWiringName::ConsumerTraitAsKey {
        consumer: tcx.item_name(consumer_did).to_string(),
        component: tcx.item_name(component_did).to_string(),
    })
}

/// The action for each name-resolution `E0425` "cannot find type `name`", given as its primary span
/// and the name, in the order given. A name outside a wiring entry or a `#[cgp_impl]` header gets
/// `None` and stays as the compiler wrote it, repeats included. Within them, a repeat of an earlier
/// diagnostic at the same span is dropped, a wiring value naming the provider a `#[cgp_impl]` header
/// also fails to declare is dropped as that header's consequence, and a name no rule explains gets
/// `None`. A type inside an `@`-path of a key or value is a component that needs an import or a type
/// never declared, and a later one of the same name is dropped as a repeat. A value, a `#[cgp_impl]`
/// provider, or a path type that names a type declared elsewhere was not imported, so it gets `None`.
pub fn plan_unresolved_names(
    tcx: TyCtxt<'_>,
    candidates: &[(Span, String)],
) -> UnresolvedNamesPlan {
    let slots: Vec<Option<NameSlot<'_>>> = candidates
        .iter()
        .map(|(span, _)| name_slot_at(tcx, *span))
        .collect();
    let mut declared = DeclaredNames::default();
    let undeclared_in_cgp_impl: Vec<&str> = candidates
        .iter()
        .zip(&slots)
        .filter(|(_, slot)| matches!(slot, Some(NameSlot::CgpImplProvider)))
        .map(|((_, name), _)| name.as_str())
        .filter(|name| !declared.is_declared(tcx, name))
        .collect();
    let mut markers: Option<Vec<(DefId, DefId)>> = None;
    let mut overflow_requirements = Vec::new();
    let mut reported_in_paths: Vec<&str> = Vec::new();

    let actions = candidates
        .iter()
        .zip(&slots)
        .enumerate()
        .map(|(index, ((span, name), slot))| {
            let slot = slot.as_ref()?;
            let repeat = candidates[..index]
                .iter()
                .any(|(earlier_span, earlier_name)| earlier_span == span && earlier_name == name);
            if repeat {
                return Some(NameAction::Drop);
            }
            match slot {
                NameSlot::Key { .. } => {
                    let markers = markers.get_or_insert_with(|| component_markers(tcx));
                    unresolved_key(tcx, name, markers).map(NameAction::Reword)
                }
                NameSlot::Value { key, impl_did } => {
                    overflow_requirements.extend(unprovable_requirements(tcx, key, *impl_did));
                    if undeclared_in_cgp_impl.contains(&name.as_str()) {
                        return Some(NameAction::Drop);
                    }
                    if declared.is_declared(tcx, name) {
                        return None;
                    }
                    let markers = markers.get_or_insert_with(|| component_markers(tcx));
                    let key_text = tcx.sess.source_map().span_to_snippet(key.span).ok()?;
                    let near_miss = providers_of_key(tcx, key, markers).and_then(|providers| {
                        similar_name(name, providers.iter().map(String::as_str)).map(str::to_owned)
                    });
                    Some(NameAction::Reword(
                        UnresolvedWiringName::ProviderNotDeclared {
                            provider: name.clone(),
                            key: key_text,
                            near_miss,
                        },
                    ))
                }
                NameSlot::CgpImplProvider => (!declared.is_declared(tcx, name)).then(|| {
                    NameAction::Reword(UnresolvedWiringName::CgpImplProviderNotDeclared {
                        provider: name.clone(),
                    })
                }),
                NameSlot::Inside {
                    key,
                    impl_did,
                    in_value,
                    whole_segment,
                } => {
                    if *in_value {
                        overflow_requirements.extend(unprovable_requirements(tcx, key, *impl_did));
                    }
                    if reported_in_paths.contains(&name.as_str()) {
                        return Some(NameAction::Drop);
                    }
                    let markers = markers.get_or_insert_with(|| component_markers(tcx));
                    if *whole_segment
                        && let Some(not_imported) = component_not_imported(tcx, name, markers)
                    {
                        reported_in_paths.push(name.as_str());
                        return Some(NameAction::Reword(not_imported));
                    }
                    if declared.is_declared(tcx, name) {
                        return None;
                    }
                    reported_in_paths.push(name.as_str());
                    Some(NameAction::RewordHeader(
                        UnresolvedWiringName::PathTypeNotDeclared {
                            name: name.clone(),
                            whole_segment: *whole_segment,
                        },
                    ))
                }
            }
        })
        .collect();
    UnresolvedNamesPlan {
        actions,
        overflow_requirements,
    }
}

/// The requirements a wiring entry with an unresolved value leaves unprovable, as the compiler
/// prints them up to the component's parameters: the entry's context implementing the consumer
/// trait of the component its key names (`Person: CanGreet`, what a call reports), and using the
/// component (`Person: CanUseComponent<GreeterComponent`, what a check reports, closed by `>` or
/// followed by the parameters it checks).
fn unprovable_requirements(
    tcx: TyCtxt<'_>,
    key: &rustc_hir::Ty<'_>,
    impl_did: rustc_hir::def_id::LocalDefId,
) -> Vec<String> {
    let TyKind::Path(QPath::Resolved(None, path)) = key.kind else {
        return Vec::new();
    };
    let Some(marker_did) = path.res.opt_def_id() else {
        return Vec::new();
    };
    let marker = tcx
        .type_of(marker_did)
        .instantiate_identity()
        .skip_norm_wip();
    let Some((consumer_did, _)) = marker_to_consumer(tcx, marker) else {
        return Vec::new();
    };
    let context = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
    vec![
        format!("{context}: {}", tcx.def_path_str(consumer_did)),
        format!(
            "{context}: CanUseComponent<{}",
            tcx.def_path_str(marker_did)
        ),
    ]
}

/// A wiring key `name` that does not resolve: the one component of that name elsewhere, which needs
/// an import, or, when `name` is a consumer trait's name with `Component` appended, that trait's
/// component, named differently.
fn unresolved_key(
    tcx: TyCtxt<'_>,
    name: &str,
    markers: &[(DefId, DefId)],
) -> Option<UnresolvedWiringName> {
    if is_component_name(tcx, name, markers) {
        return component_not_imported(tcx, name, markers);
    }
    let consumer = name.strip_suffix("Component")?;
    let consumer_did = consumer_trait_named(tcx, consumer)?;
    let component_did = consumer_component(tcx, consumer_did)?;
    Some(UnresolvedWiringName::KeyNamesNoComponent {
        key: name.to_owned(),
        consumer: consumer.to_owned(),
        component: tcx.item_name(component_did).to_string(),
    })
}

/// The names of the providers that implement the provider trait of the component `key` names, when
/// it names one.
fn providers_of_key(
    tcx: TyCtxt<'_>,
    key: &rustc_hir::Ty<'_>,
    markers: &[(DefId, DefId)],
) -> Option<Vec<String>> {
    let TyKind::Path(QPath::Resolved(None, path)) = key.kind else {
        return None;
    };
    let key_did = path.res.opt_def_id()?;
    let &(_, provider_trait) = markers
        .iter()
        .find(|(marker_did, _)| *marker_did == key_did)?;
    let impls = tcx.trait_impls_of(provider_trait);
    Some(
        impls
            .non_blanket_impls()
            .values()
            .flatten()
            .filter_map(
                |&impl_did| match tcx.type_of(impl_did).skip_binder().kind() {
                    ty::Adt(provider_def, _) => Some(tcx.item_name(provider_def.did()).to_string()),
                    _ => None,
                },
            )
            .collect(),
    )
}

/// Whether some component marker is named `name`.
fn is_component_name(tcx: TyCtxt<'_>, name: &str, markers: &[(DefId, DefId)]) -> bool {
    markers
        .iter()
        .any(|(marker_did, _)| tcx.item_name(*marker_did).as_str() == name)
}

/// The component `name` names, when exactly one marker is so named, as one that needs an import.
/// `None` when no marker, several markers, or no public path to the one, so the import is ambiguous.
fn component_not_imported(
    tcx: TyCtxt<'_>,
    name: &str,
    markers: &[(DefId, DefId)],
) -> Option<UnresolvedWiringName> {
    let mut named = markers
        .iter()
        .filter(|(marker_did, _)| tcx.item_name(*marker_did).as_str() == name);
    let &(marker_did, _) = named.next()?;
    if named.next().is_some() {
        return None;
    }
    Some(UnresolvedWiringName::ComponentNotImported {
        component: name.to_owned(),
        path: import_path(tcx, marker_did)?,
    })
}

/// Whether a type of each name asked about is declared where the crate could import it from,
/// remembered per name, since the answer walks the crate and its dependencies.
#[derive(Default)]
struct DeclaredNames(Vec<(String, bool)>);

impl DeclaredNames {
    fn is_declared(&mut self, tcx: TyCtxt<'_>, name: &str) -> bool {
        if let Some(&(_, declared)) = self.0.iter().find(|(known, _)| known == name) {
            return declared;
        }
        let declared = is_type_declared(tcx, name);
        self.0.push((name.to_owned(), declared));
        declared
    }
}
