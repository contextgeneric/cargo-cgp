//! A check table that checks one entry twice.

use cargo_cgp_error_processing::{CheckForm, StructuralMistake};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use super::check_macro_at;
use crate::resolve::query::{can_use_component_supertrait, local_trait_impls, same_range};

/// What an `E0119` between two identical check impls is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateCheck {
    /// A `check_components!` table that lists one entry twice.
    Entry(StructuralMistake),
    /// A `delegate_and_check_components!` table that lists one entry twice. The same entry also
    /// wires its key twice, which the wiring-conflict stage reports as `[CGP-E010]`, so the check
    /// impls' conflict restates that mistake.
    WiredTwice,
}

/// The duplicate check entry an `E0119` at `primary_span` is about: a local impl of a check trait
/// (one whose supertrait is `CanUseComponent`) whose def-span is the caret, which rustc aims at the
/// conflicting impl. The two impls a repeated entry generates are identical, so the entry is read
/// off the one at the caret, and the macro that wrote it says whether the wiring repeats too.
/// `None` when no check impl is there.
pub fn detect_duplicate_check(tcx: TyCtxt<'_>, primary_span: Span) -> Option<DuplicateCheck> {
    for local in local_trait_impls(tcx) {
        let impl_did = local.to_def_id();
        if !same_range(tcx.def_span(impl_did), primary_span) {
            continue;
        }
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        if can_use_component_supertrait(tcx, trait_ref.def_id).is_none() {
            continue;
        }
        if check_macro_at(tcx.def_span(impl_did)) == Some(CheckForm::DelegateAndCheckComponents) {
            return Some(DuplicateCheck::WiredTwice);
        }
        // A check trait is `__Check{Context}<__Component__, __Params__>`, so its arguments are
        // `[Context, Component, Params]`; a component without parameters is checked at `()`.
        let marker = trait_ref.args.type_at(1);
        let params = trait_ref.args.type_at(2);
        let entry = if params.is_unit() {
            marker.to_string()
        } else {
            format!("{marker}: {params}")
        };
        return Some(DuplicateCheck::Entry(StructuralMistake::DuplicateCheck {
            context: trait_ref.self_ty().to_string(),
            entry,
        }));
    }
    None
}
