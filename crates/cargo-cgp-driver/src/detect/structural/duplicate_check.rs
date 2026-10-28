//! A `check_components!` table that checks one entry twice.

use cargo_cgp_error_processing::StructuralMistake;
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::detect::conflict::same_range;
use crate::resolve::query::can_use_component_supertrait;

/// The duplicate check entry an `E0119` at `primary_span` is about: a local impl of a
/// `check_components!` check trait (one whose supertrait is `CanUseComponent`) whose def-span is
/// the caret, which rustc aims at the conflicting impl. The two impls a repeated entry generates
/// are identical, so the entry is read off the one at the caret. `None` when no check impl is there.
pub fn detect_duplicate_check(tcx: TyCtxt<'_>, primary_span: Span) -> Option<StructuralMistake> {
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
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
        // A check trait is `__Check{Context}<__Component__, __Params__>`, so its arguments are
        // `[Context, Component, Params]`; a component without parameters is checked at `()`.
        let marker = trait_ref.args.type_at(1);
        let params = trait_ref.args.type_at(2);
        let entry = if params.is_unit() {
            marker.to_string()
        } else {
            format!("{marker}: {params}")
        };
        return Some(StructuralMistake::DuplicateCheck {
            context: trait_ref.self_ty().to_string(),
            entry,
        });
    }
    None
}
