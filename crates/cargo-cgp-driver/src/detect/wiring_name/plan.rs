//! Planning what to do with the name-resolution errors of the wiring entries and `#[cgp_impl]`
//! headers.

use cargo_cgp_error_processing::{UnresolvedWiringName, similar_name};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::{DefId, LocalDefId};

use crate::detect::wiring_name::component::component_markers;
use crate::detect::wiring_name::slot::{NameSlot, name_slot_at};
use crate::detect::wiring_name::{
    DeclaredNames, component_not_imported, providers_of_key, unprovable_requirements,
    unresolved_key,
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
    /// Reword only its header into the coded form of this mistake, keeping the compiler's own notes
    /// and suggestions, which are right for the mistake except for a type parameter to add.
    RewordHeader(UnresolvedWiringName),
    /// Drop it: a repeat of a diagnostic at the same span, or the consequence of another's mistake.
    Drop,
}

/// The action for each name-resolution `E0425` "cannot find type `name`", given as its primary span
/// and the name, in the order given. A name outside a wiring entry or a `#[cgp_impl]` header gets
/// `None` and stays as the compiler wrote it, repeats included. Within them, a repeat of an earlier
/// diagnostic at the same span is dropped, a wiring value naming the provider a `#[cgp_impl]`
/// header also fails to declare is dropped as that header's consequence, and a name no rule
/// explains gets `None`. A type inside an `@`-path of a key or value is a component that needs an
/// import or a type never declared, and a later one of the same name is dropped as a repeat. A
/// value, a `#[cgp_impl]` provider, or a path type that names a type declared elsewhere was not
/// imported, so it gets `None`.
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
        .map(|(index, ((span, name), slot))| {
            let slot = slot.as_ref()?;
            let repeat = candidates[..index]
                .iter()
                .any(|(earlier_span, earlier_name)| earlier_span == span && earlier_name == name);
            if repeat {
                return Some(NameAction::Drop);
            }
            match slot {
                NameSlot::Key { .. } => planner.key_action(name),
                NameSlot::Value { key, impl_did } => planner.value_action(name, key, *impl_did),
                NameSlot::CgpImplProvider => planner.cgp_impl_action(name),
                NameSlot::Inside {
                    key,
                    impl_did,
                    in_value,
                    whole_segment,
                } => planner.inside_action(name, key, *impl_did, *in_value, *whole_segment),
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
    /// A name that is the key of a wiring entry.
    fn key_action(&mut self, name: &str) -> Option<NameAction> {
        let tcx = self.tcx;
        let markers = self.markers.get_or_insert_with(|| component_markers(tcx));
        unresolved_key(tcx, name, markers).map(NameAction::Reword)
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

    /// A name inside the `@`-path of a key or value.
    fn inside_action(
        &mut self,
        name: &'a str,
        key: &'tcx rustc_hir::Ty<'tcx>,
        impl_did: LocalDefId,
        in_value: bool,
        whole_segment: bool,
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
            return Some(NameAction::Reword(not_imported));
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
