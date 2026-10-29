//! Reading the type a requirement gives a field, from the impl that requires it.

use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::ty::{self, TyCtxt, TypeVisitableExt, TypingMode};
use rustc_trait_selection::traits::ObligationCtxt;

use crate::resolve::query::{impls_concrete_first, match_impl, misdirected_impl_clauses};

/// The type the impl satisfying `parent` requires `leaf` (a `HasField` bound) to project, read off
/// the associated-type constraint beside the bound (the `f64` of
/// `HasField<Symbol!("height"), Value = f64>`, as a getter or an `#[implicit]` argument writes it).
/// A provider whose impl names the consumer trait in its header has no impl satisfying `parent`, so
/// that impl's own clauses are read instead ([`misdirected_impl_clauses`]). `None` when no impl
/// matches `parent`, no constraint names `leaf`, or the type it names is not concrete, so a fix
/// never names a type the program could not write.
pub fn required_field_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    parent: ty::TraitRef<'tcx>,
    leaf: ty::TraitRef<'tcx>,
) -> Option<String> {
    if parent.has_escaping_bound_vars() || leaf.has_escaping_bound_vars() {
        return None;
    }
    let leaf = tcx.erase_and_anonymize_regions(leaf);
    if let Some(clauses) = misdirected_impl_clauses(tcx, parent) {
        return required_type_in(tcx, clauses, leaf);
    }
    for impl_did in impls_concrete_first(tcx, parent.def_id) {
        let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
        let ocx = ObligationCtxt::new(&infcx);
        // `parent` carries no bound variables (checked above), so its binder is a dummy.
        let Some(impl_args) = match_impl(&ocx, ty::Binder::dummy(parent), impl_did) else {
            continue;
        };
        let clauses = tcx
            .clauses_of(impl_did)
            .instantiate(tcx, impl_args)
            .into_iter()
            .map(|(clause, _)| infcx.deeply_resolve_ignoring_regions(clause.skip_norm_wip()));
        return required_type_in(tcx, clauses, leaf);
    }
    None
}

/// The type the constraint among an impl's instantiated `clauses` requires the region-erased `leaf`
/// to project, when that type is concrete.
fn required_type_in<'tcx>(
    tcx: TyCtxt<'tcx>,
    clauses: impl IntoIterator<Item = ty::Clause<'tcx>>,
    leaf: ty::TraitRef<'tcx>,
) -> Option<String> {
    for clause in clauses {
        if clause.has_non_region_infer() {
            continue;
        }
        let Some(projection) = tcx
            .erase_and_anonymize_regions(clause)
            .as_projection_clause()
        else {
            continue;
        };
        let projection = projection.skip_binder();
        if !matches!(
            projection.projection_term.kind,
            ty::AliasTermKind::ProjectionTy { .. }
        ) || projection.projection_term.trait_ref(tcx) != leaf
        {
            continue;
        }
        let required = projection.term.as_type()?;
        if required.has_param() || required.has_placeholders() || required.has_aliases() {
            return None;
        }
        return Some(required.to_string());
    }
    None
}
