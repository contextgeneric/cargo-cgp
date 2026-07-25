//! Surfacing an associated-type mismatch the trait-clause walk cannot see.

use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt, TypingMode, Unnormalized};
use rustc_span::DUMMY_SP;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::config::{CGP_FIELD_CRATE, HAS_FIELD_TRAIT};
use crate::resolve::cgp_item::is_cgp_item;
use crate::resolve::walk::{holds_projection, impls_concrete_first};

/// An unmet associated-type projection carried by the impl that would satisfy an obligation —
/// `<Ctx as HasField<Symbol!("height")>>::Value == f64`, or `<Ctx as HasErrorType>::Error ==
/// AppError`. The trait bound itself holds, so the walk reaches it only through the branch where
/// every trait-clause dependency held.
#[derive(Clone, Copy)]
pub(crate) struct ProjectionMismatch<'tcx> {
    /// The trait ref the projection is taken on (`Ctx: HasErrorType`) — the terminal the tree shows.
    pub(crate) trait_ref: ty::TraitRef<'tcx>,
    /// The associated item being projected (`Error`), so the leaf can name it.
    pub(crate) assoc_did: DefId,
    /// The projected type `<Ctx as HasErrorType>::Error` as a non-rigid alias, region-erased and
    /// free of inference variables, so the leaf can normalize it to the type the owner supplies.
    pub(crate) alias: Ty<'tcx>,
    /// The type the projection is required to equal (`AppError`).
    pub(crate) expected: Ty<'tcx>,
}

/// When the impl that satisfies `pred`'s trait obligation carries an unmet associated-type
/// projection, return it. This covers two shapes of the same failure: a `HasField` value type (a
/// field present with the wrong type) and any other associated type — most often a CGP [abstract
/// type](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/abstract-types.md)
/// a provider pinned with the `#[use_type(Trait.{Assoc = Concrete})]` equality form while the
/// context bound it to something else. `None` when the impl carries no unmet projection.
///
/// A `HasField` projection wins when the impl carries both, because a field's value type is the more
/// specific classification and the one whose leaf can name the struct field to fix; the walk reports
/// a single leaf per node either way, as it always has.
///
/// Mirrors [`impl_where_obligations`](super::impl_where_obligations)'s next-solver-safe impl match
/// (`fresh_args_for_item` + `eq`), but keeps the projection predicates rather than the trait ones,
/// and leaves each projection un-normalized so its `<.. as Trait>::Assoc` alias survives for the
/// hold check.
pub(crate) fn projection_mismatch<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
) -> Option<ProjectionMismatch<'tcx>> {
    // Prefer a concrete-`Self` impl, falling back to a blanket, exactly as
    // `impl_where_obligations` does. For a provider trait both the concrete provider impl (which
    // carries the projection) and the delegation blanket (`impl<.., P> Provider<..> for P`, which
    // does not) unify with the obligation, and only the concrete one is authoritative — matching the
    // blanket first would wrongly report no mismatch. But a getter trait's *only* impl is itself a
    // blanket (`impl<C: HasField<..>> HasName for C`), and that one *does* carry the projection — so
    // a blanket is deferred, not skipped outright.
    for impl_did in impls_concrete_first(tcx, pred.def_id()) {
        if let Some(result) = impl_projection_mismatch(tcx, pred, impl_did) {
            return result;
        }
    }
    None
}

/// Test one impl for [`projection_mismatch`]: `None` when the impl does not unify with `pred`,
/// `Some(None)` when it unifies but carries no unmet projection, and `Some(Some(mismatch))` for the
/// unmet projection it does carry.
fn impl_projection_mismatch<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    impl_did: DefId,
) -> Option<Option<ProjectionMismatch<'tcx>>> {
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

    let mut fallback: Option<ProjectionMismatch<'tcx>> = None;
    for (predicate, _) in tcx.predicates_of(impl_did).instantiate(tcx, impl_args) {
        // Keep the projection un-normalized so its `<.. as Trait>::Assoc` alias survives;
        // `skip_norm_wip` unwraps without normalizing, unlike the `ocx.normalize` the trait-clause
        // walk uses.
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
        let projection = proj.skip_binder().projection_term;
        // Only a trait's associated *type* projection is classifiable here — an opaque, inherent, or
        // const alias has no `Trait::Assoc` to name, and the accessors below would panic on one.
        if !matches!(projection.kind, ty::AliasTermKind::ProjectionTy { .. }) {
            continue;
        }
        let Some(expected) = proj.skip_binder().term.as_type() else {
            continue;
        };
        // The solver query is the expensive step, so it runs last, after the cheap structural
        // filters have discarded everything that could not be a reportable mismatch anyway.
        if holds_projection(tcx, proj) {
            continue;
        }
        let mismatch = ProjectionMismatch {
            trait_ref: projection.trait_ref(tcx),
            assoc_did: projection.expect_projection_def_id(),
            // Built non-rigid so the leaf's normalization is allowed to reduce it; a rigid alias
            // would come back unchanged under the next-generation solver the driver runs.
            alias: Ty::new_alias(tcx, ty::IsRigid::No, projection.expect_ty()),
            expected,
        };
        // A `HasField` value type is the more specific classification, so it wins outright; any
        // other associated type is remembered and reported only if no `HasField` one turns up.
        if is_cgp_item(
            tcx,
            mismatch.trait_ref.def_id,
            HAS_FIELD_TRAIT,
            CGP_FIELD_CRATE,
        ) {
            return Some(Some(mismatch));
        }
        fallback.get_or_insert(mismatch);
    }
    // Unified with the impl; report whatever non-`HasField` mismatch it carried, if any.
    Some(fallback)
}
