//! Planning what to do with the name-resolution errors of the wiring entries and `#[cgp_impl]`
//! headers.

use cargo_cgp_error_processing::{UnresolvedWiringName, similar_name};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::{DefId, LocalDefId};

use crate::detect::wiring_name::component::component_markers;
use crate::detect::wiring_name::slot::{NameSlot, name_slot_at};
use crate::detect::wiring_name::{
    DeclaredNames, NameFix, component_not_imported, derived_component_fix,
    header_component_mistake, import_fix, named_marker, provider_header_at, providers_of_key,
    unprovable_requirements, unresolved_key,
};

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
    /// Reword it into the coded form of this mistake, with its help given as a fix: the import of
    /// a component not in scope, or the component named in a provider header's attribute.
    Fix(UnresolvedWiringName, NameFix),
    /// Reword only its header into the coded form of this mistake, keeping the compiler's own notes
    /// and suggestions, which are right for the mistake except for a type parameter to add.
    RewordHeader(UnresolvedWiringName),
    /// Drop it: a repeat of a diagnostic at the same span, or the consequence of another's mistake.
    Drop,
}

/// One name-resolution `E0425` "cannot find type `name`": its primary span, the name, and the module
/// rustc says lacks it, for the ``in module `path` `` form a qualified path gets.
pub struct UnresolvedNameCandidate {
    /// The diagnostic's primary span, the name as written.
    pub span: Span,
    /// The name rustc cannot find.
    pub name: String,
    /// The module rustc looked in, for a qualified path.
    pub module: Option<String>,
}

/// The action for each name-resolution `E0425` "cannot find type `name`", in the order given. A
/// name outside a wiring entry, a `#[cgp_impl]` header's provider, or a provider header's derived
/// component gets `None` and stays as the compiler wrote it, repeats included. A provider header's
/// derived component is one not imported, not re-exported by the module the header names, or not
/// the trait's component at all. Only that position takes the ``in module`` form, since a path
/// written in a wiring entry is the reader's own. Within them, a repeat of an earlier
/// diagnostic at the same span is dropped, a wiring value naming the provider a `#[cgp_impl]`
/// header also fails to declare is dropped as that header's consequence, and a name no rule
/// explains gets `None`. A type inside an `@`-path of a key or value is a component that needs an
/// import or a type never declared, and a later one of the same name is dropped as a repeat. A
/// value, a `#[cgp_impl]` provider, or a path type that names a type declared elsewhere was not
/// imported, so it gets `None`.
pub fn plan_unresolved_names(
    tcx: TyCtxt<'_>,
    candidates: &[UnresolvedNameCandidate],
) -> UnresolvedNamesPlan {
    let slots: Vec<Option<NameSlot<'_>>> = candidates
        .iter()
        .map(|candidate| {
            candidate
                .module
                .is_none()
                .then(|| name_slot_at(tcx, candidate.span))
                .flatten()
        })
        .collect();
    let mut declared = DeclaredNames::default();
    let undeclared_in_cgp_impl: Vec<&str> = candidates
        .iter()
        .zip(&slots)
        .filter(|(_, slot)| matches!(slot, Some(NameSlot::CgpImplProvider)))
        .map(|(candidate, _)| candidate.name.as_str())
        .filter(|name| !declared.is_declared(tcx, name))
        .collect();
    let mut planner = NamesPlanner {
        tcx,
        undeclared_in_cgp_impl,
        declared,
        markers: None,
        overflow_requirements: Vec::new(),
        reported_in_paths: Vec::new(),
    };

    let actions = candidates
        .iter()
        .zip(&slots)
        .enumerate()
        .map(|(index, (candidate, slot))| {
            let UnresolvedNameCandidate { span, name, module } = candidate;
            let repeat = candidates[..index].iter().any(|earlier| {
                earlier.span == *span && earlier.name == *name && earlier.module == *module
            });
            let Some(slot) = slot else {
                let header = provider_header_at(tcx, *span)?;
                if repeat {
                    return Some(NameAction::Drop);
                }
                let mistake = header_component_mistake(tcx, &header, name, module.as_deref())?;
                let fix = match mistake {
                    UnresolvedWiringName::DerivedComponentName { .. } => {
                        derived_component_fix(tcx, &header, &mistake, *span)
                    }
                    _ => import_fix(tcx, header.impl_did, header.marker, &mistake, *span),
                };
                return Some(with_fix(mistake, fix));
            };
            if repeat {
                return Some(NameAction::Drop);
            }
            match slot {
                NameSlot::Key { impl_did, .. } => planner.key_action(name, *impl_did, *span),
                NameSlot::Value { key, impl_did } => planner.value_action(name, key, *impl_did),
                NameSlot::CgpImplProvider => planner.cgp_impl_action(name),
                NameSlot::Inside {
                    key,
                    impl_did,
                    in_value,
                    whole_segment,
                } => planner.inside_action(name, key, *impl_did, *in_value, *whole_segment, *span),
            }
        })
        .collect();
    UnresolvedNamesPlan {
        actions,
        overflow_requirements: planner.overflow_requirements,
    }
}

/// What the candidates of one plan share: the names and markers read once, the requirements to
/// expect, and the path types already reported.
struct NamesPlanner<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    undeclared_in_cgp_impl: Vec<&'a str>,
    declared: DeclaredNames,
    markers: Option<Vec<(DefId, DefId)>>,
    overflow_requirements: Vec<String>,
    reported_in_paths: Vec<&'a str>,
}

impl<'a, 'tcx> NamesPlanner<'a, 'tcx> {
    /// A name that is the key of the wiring entry `impl_did`, at `span`.
    fn key_action(&mut self, name: &str, impl_did: LocalDefId, span: Span) -> Option<NameAction> {
        let tcx = self.tcx;
        let markers = self.markers.get_or_insert_with(|| component_markers(tcx));
        let mistake = unresolved_key(tcx, name, markers)?;
        let marker = named_marker(tcx, name, markers);
        let fix = marker.and_then(|marker| import_fix(tcx, impl_did, marker, &mistake, span));
        Some(with_fix(mistake, fix))
    }

    /// A name that is the value of a wiring entry.
    fn value_action(
        &mut self,
        name: &str,
        key: &'tcx rustc_hir::Ty<'tcx>,
        impl_did: LocalDefId,
    ) -> Option<NameAction> {
        let tcx = self.tcx;
        self.overflow_requirements
            .extend(unprovable_requirements(tcx, key, impl_did));
        if self.undeclared_in_cgp_impl.contains(&name) {
            return Some(NameAction::Drop);
        }
        if self.declared.is_declared(tcx, name) {
            return None;
        }
        let markers = self.markers.get_or_insert_with(|| component_markers(tcx));
        let key_text = tcx.sess.source_map().span_to_snippet(key.span).ok()?;
        let near_miss = providers_of_key(tcx, key, markers).and_then(|providers| {
            similar_name(name, providers.iter().map(String::as_str)).map(str::to_owned)
        });
        Some(NameAction::Reword(
            UnresolvedWiringName::ProviderNotDeclared {
                provider: name.to_owned(),
                key: key_text,
                near_miss,
            },
        ))
    }

    /// A name that is the provider of a `#[cgp_impl]` header.
    fn cgp_impl_action(&mut self, name: &str) -> Option<NameAction> {
        (!self.declared.is_declared(self.tcx, name)).then(|| {
            NameAction::Reword(UnresolvedWiringName::CgpImplProviderNotDeclared {
                provider: name.to_owned(),
            })
        })
    }

    /// A name at `span` inside the `@`-path of a key or value.
    fn inside_action(
        &mut self,
        name: &'a str,
        key: &'tcx rustc_hir::Ty<'tcx>,
        impl_did: LocalDefId,
        in_value: bool,
        whole_segment: bool,
        span: Span,
    ) -> Option<NameAction> {
        let tcx = self.tcx;
        if in_value {
            self.overflow_requirements
                .extend(unprovable_requirements(tcx, key, impl_did));
        }
        if self.reported_in_paths.contains(&name) {
            return Some(NameAction::Drop);
        }
        let markers = self.markers.get_or_insert_with(|| component_markers(tcx));
        if whole_segment && let Some(not_imported) = component_not_imported(tcx, name, markers) {
            self.reported_in_paths.push(name);
            let fix = named_marker(tcx, name, markers)
                .and_then(|marker| import_fix(tcx, impl_did, marker, &not_imported, span));
            return Some(with_fix(not_imported, fix));
        }
        if self.declared.is_declared(tcx, name) {
            return None;
        }
        self.reported_in_paths.push(name);
        Some(NameAction::RewordHeader(
            UnresolvedWiringName::PathTypeNotDeclared {
                name: name.to_owned(),
                whole_segment,
            },
        ))
    }
}

/// The action for the mistake `name`: its coded form with `fix` in place of its help, or with its
/// help when there is no fix.
fn with_fix(name: UnresolvedWiringName, fix: Option<NameFix>) -> NameAction {
    match fix {
        Some(fix) => NameAction::Fix(name, fix),
        None => NameAction::Reword(name),
    }
}
