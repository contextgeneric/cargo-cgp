//! Surfacing a field-type mismatch the trait-clause walk cannot see.

use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt, TypingMode, Unnormalized};
use rustc_span::DUMMY_SP;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::config::{CGP_FIELD_CRATE, HAS_FIELD_TRAIT};
use crate::resolve::cgp_item::is_cgp_item;
use crate::resolve::walk::{holds_projection, impls_concrete_first};

/// When the impl that satisfies `pred`'s trait obligation carries an unmet `HasField`
/// associated-type projection — `<Ctx as HasField<Symbol!("height")>>::Value == f64` — return that
/// field's `HasField` trait ref (the terminal the tree shows) paired with the expected type
/// (`f64`). This is the field-present-with-wrong-type case: the trait bound holds, so the walk
/// reaches it only here, in the branch where every trait-clause dependency held. `None` when the
/// impl carries no such unmet `HasField` projection.
///
/// Mirrors [`impl_where_obligations`](super::impl_where_obligations)'s next-solver-safe impl match
/// (`fresh_args_for_item` + `eq`), but keeps the projection predicates rather than the trait ones,
/// and leaves each projection un-normalized so its `<.. as HasField<..>>::Value` alias survives
/// for the hold check.
pub(crate) fn has_field_projection_mismatch<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
) -> Option<(ty::TraitRef<'tcx>, Ty<'tcx>)> {
    // Prefer a concrete-`Self` impl, falling back to a blanket, exactly as
    // `impl_where_obligations` does. For a provider trait both the concrete provider impl (which
    // carries the `HasField` projection) and the delegation blanket (`impl<.., P> Provider<..> for
    // P`, which does not) unify with the obligation, and only the concrete one is authoritative —
    // matching the blanket first would wrongly report no mismatch. But a getter trait's *only* impl
    // is itself a blanket (`impl<C: HasField<..>> HasName for C`), and that one *does* carry the
    // projection — so a blanket is deferred, not skipped outright.
    for impl_did in impls_concrete_first(tcx, pred.def_id()) {
        if let Some(result) = impl_field_projection_mismatch(tcx, pred, impl_did) {
            return result;
        }
    }
    None
}

/// Test one impl for [`has_field_projection_mismatch`]: `None` when the impl does not unify with
/// `pred`, `Some(None)` when it unifies but carries no unmet `HasField` projection, and
/// `Some(Some((field_ref, expected)))` for the field-present-with-wrong-type mismatch it does carry.
fn impl_field_projection_mismatch<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    impl_did: DefId,
) -> Option<Option<(ty::TraitRef<'tcx>, Ty<'tcx>)>> {
    let param_env = ty::ParamEnv::empty();
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);

    // Instantiate any higher-ranked binder with placeholders before relating, for the same reason
    // as `impl_where_obligations`: a `skip_binder()`'d escaping bound var fed into `ocx.eq` panics
    // rustc's generalizer. A no-op for a binder-free predicate.
    let obligation_ref = infcx.enter_forall_and_leak_universe(pred.map_bound(|p| p.trait_ref));

    let impl_args = infcx.fresh_args_for_item(DUMMY_SP, impl_did);
    let impl_ref = tcx
        .impl_trait_ref(impl_did)
        .instantiate(tcx, impl_args)
        .skip_norm_wip();
    let impl_ref = ocx.normalize(
        &ObligationCause::dummy(),
        param_env,
        Unnormalized::new_wip(impl_ref),
    );
    if ocx
        .eq(
            &ObligationCause::dummy(),
            param_env,
            obligation_ref,
            impl_ref,
        )
        .is_err()
    {
        return None;
    }

    for (predicate, _) in tcx.predicates_of(impl_did).instantiate(tcx, impl_args) {
        // Keep the projection un-normalized so its `<.. as HasField<..>>::Value` alias
        // survives; `skip_norm_wip` unwraps without normalizing, unlike the `ocx.normalize`
        // the trait-clause walk uses.
        let clause = infcx.resolve_vars_if_possible(predicate.skip_norm_wip());
        // An unconstrained impl parameter leaves inference vars behind; such a projection
        // cannot be re-checked in a fresh context, so skip it (regions are erased below).
        if clause.has_non_region_infer() {
            continue;
        }
        let Some(proj) = tcx
            .erase_and_anonymize_regions(clause)
            .as_projection_clause()
        else {
            continue;
        };
        let field_ref = proj.skip_binder().projection_term.trait_ref(tcx);
        if !is_cgp_item(tcx, field_ref.def_id, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) {
            continue;
        }
        if holds_projection(tcx, proj) {
            continue;
        }
        let Some(expected) = proj.skip_binder().term.as_type() else {
            continue;
        };
        return Some(Some((field_ref, expected)));
    }
    // Unified with the impl but found no unmet `HasField` projection on it.
    Some(None)
}
