//! Finding the impl that satisfies an obligation and reading its dependencies.

use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::Obligation;
use rustc_middle::ty::{self, TyCtxt, TypeVisitableExt, TypingMode, Unnormalized};
use rustc_span::DUMMY_SP;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::walk::{resolve_fixed_projections, unknowns_to_placeholders};

/// The instantiated `where`-clause trait obligations of the impl that would satisfy `obligation`
/// — its direct dependencies — or `None` when no impl matches at all (so the caller can treat the
/// obligation as a terminal leaf). Found by unifying `obligation` with each candidate impl's trait
/// ref (the next-solver-safe `fresh_args_for_item` + `eq` dance `SelectionContext` is unavailable
/// for), then instantiating and normalizing that impl's predicates. `Some(vec![])` means an impl
/// matched but carries no trait-clause `where` obligations.
///
/// A **concrete-`Self`** impl (one whose declared `Self` is a struct/enum, like the `#[cgp_provider]`
/// impl `impl ValueDeserializer<…> for DeserializeRecordFields`) is preferred over a **blanket** one
/// (whose `Self` is a bare type parameter, like the CGP delegation blanket `impl<P: DelegateComponent>
/// ValueDeserializer<…> for P`). Both unify with a provider obligation such as
/// `DeserializeRecordFields: ValueDeserializer<…>`, but only the specific impl's `where`-clauses lead
/// to the real cause; the blanket's lead to a `DeserializeRecordFields: DelegateComponent`
/// dead-end, since a leaf provider does not delegate. A blanket impl is used only when no
/// concrete-`Self` one matches — the usual case for an obligation whose `Self` *is* the context
/// (`App: CanUseComponent<…>` has only the blanket).
pub(crate) fn impl_where_obligations<'tcx>(
    tcx: TyCtxt<'tcx>,
    obligation: ty::PolyTraitPredicate<'tcx>,
) -> Option<Vec<ty::PolyTraitPredicate<'tcx>>> {
    let param_env = ty::ParamEnv::empty();

    for impl_did in impls_concrete_first(tcx, obligation.def_id()) {
        let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
        let ocx = ObligationCtxt::new(&infcx);

        // Instantiate the obligation's binder with placeholders in *this* infcx before it is
        // related. A higher-ranked obligation — `Self: for<'a> CanSerializeValue<&'a Value>`, the
        // shape a recursive provider like `SerializeIterator` carries — would otherwise reach `ocx.eq`
        // through `skip_binder()` with the `'a` bound var still escaping, tripping the inference
        // generalizer's `!source_term.has_escaping_bound_vars()` assertion and panicking rustc.
        // Placeholders (rigid, universal regions) rather than fresh inference vars are what let a
        // *nested* higher-ranked hop resolve: a projection through the bound lifetime (`<&'a Value as
        // IntoIterator>::Item`) normalizes deterministically against a placeholder region but stalls
        // against an unconstrained inference region. The fast path makes this a no-op for an ordinary
        // (binder-free) obligation, so only the higher-ranked case changes.
        let obligation_ref =
            infcx.enter_forall_and_leak_universe(obligation.map_bound(|p| p.trait_ref));

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
            continue;
        }

        let raw: Vec<Unnormalized<ty::Clause<'tcx>>> = tcx
            .predicates_of(impl_did)
            .instantiate(tcx, impl_args)
            .into_iter()
            .map(|(clause, _)| clause)
            .collect();
        // Register every predicate so the solver can propagate the constraints that *do* hold onto
        // the impl's otherwise-free parameters, before any single one is read. A record deserializer's
        // `Record: HasOptionalBuilder<Builder = Builder>` clause pins the free `Builder` param to the
        // concrete builder type; without solving it first, the sibling `Record::Fields:
        // HandleMapEntry<.., Builder>` clause — the branch that leads to the real cause — carries
        // `Builder` as a stray inference var and is dropped as inference-laden below.
        for &clause in &raw {
            ocx.register_obligation(Obligation::new(
                tcx,
                ObligationCause::dummy(),
                param_env,
                clause.skip_norm_wip(),
            ));
        }
        let _ = ocx.try_evaluate_obligations();

        let mut obligations = Vec::new();
        for clause in raw {
            // Best-effort: recover a *fixed* associated-type projection that stalls on an unknown
            // input before it is normalized. The canonical case is a later pipeline stage keyed on
            // an earlier stage's `::Output` (`ProviderB: Handler<Ctx, Code, <ProviderA as
            // Handler<Ctx, Code, In>>::Output>`) where the walk's input `In` is a call-site unknown
            // (a rigid placeholder): the earlier provider's `where` clause (`In: Send`, or a deeper
            // `In: Into<Body>`) is *false* against a rigid placeholder, so its impl is rejected and
            // the projection never reduces — even though the provider's `type Output` does not
            // depend on `In` at all. Resolving the projection with the placeholders as *deferrable*
            // inference variables (so the blocking bound is ambiguous, not false) lets the solver
            // commit to the sole impl and read off the fixed output, turning the next stage's input
            // from an unknowable `_` into the concrete type that carries the real cause (a later
            // stage's `stream: AsRef<[u8]>`). A no-op unless the clause carries such a placeholder.
            // Substitute the parent's unifications (including the obligation's placeholder input)
            // into the raw clause *without* normalizing, so a stalled `::Output` projection — a
            // later pipeline stage's input keyed on an earlier stage's output — survives as an
            // alias we can recover here. Normalizing first would collapse it into an opaque
            // inference variable, losing the projection.
            let pre = resolve_fixed_projections(
                tcx,
                infcx.resolve_vars_if_possible(clause.skip_norm_wip()),
            );
            let clause: ty::Clause<'tcx> = ocx.normalize(
                &ObligationCause::dummy(),
                param_env,
                Unnormalized::new_wip(pre),
            );
            let clause = infcx.resolve_vars_if_possible(clause);
            // A clause that still carries inference vars after solving is one whose parameter the
            // impl match left unconstrained — most often a *later pipeline stage keyed on an
            // earlier stage's unresolved `::Output`* (`ProviderB: Handler<Ctx, Code, ProviderA::
            // Output>`, where `ProviderA` does not hold because the walk's own input is an unknown,
            // so its `::Output` never normalizes). Dropping it would hide every root cause living
            // in a stage past the first. Instead, replace those stray vars with rigid placeholders
            // so the walk can still descend the stage; the placeholder-leaf filter in
            // [`resolve_leaves`](super::resolve_leaves) keeps any leaf that genuinely depends on
            // the unknown from being reported, exactly as for the call-site seed's unknown input. A
            // no-op when there are no such vars (the common concrete-input case), so ordinary walks
            // are unaffected.
            let clause = if clause.has_non_region_infer() {
                unknowns_to_placeholders(tcx, clause)
            } else {
                clause
            };
            if let Some(tp) = tcx.erase_and_anonymize_regions(clause).as_trait_clause() {
                obligations.push(tp);
            }
        }

        return Some(obligations);
    }
    None
}

/// Every impl of `trait_did`, with the concrete-`Self` impls ahead of the blanket (param-`Self`)
/// ones, so a caller that takes the first *unifying* impl prefers the specific impl over the
/// delegation blanket — the ordering [`impl_where_obligations`] and
/// [`has_field_projection_mismatch`](super::has_field_projection_mismatch) both rely on.
pub(crate) fn impls_concrete_first(tcx: TyCtxt<'_>, trait_did: DefId) -> Vec<DefId> {
    let (blanket, concrete): (Vec<DefId>, Vec<DefId>) =
        tcx.all_impls(trait_did).partition(|&did| {
            matches!(
                tcx.impl_trait_ref(did).skip_binder().self_ty().kind(),
                ty::Param(_)
            )
        });
    concrete.into_iter().chain(blanket).collect()
}
