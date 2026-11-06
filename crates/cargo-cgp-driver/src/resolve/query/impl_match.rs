//! Finding the impl that satisfies an obligation and reading its dependencies.

use rustc_infer::infer::{InferCtxt, TyCtxtInferExt};
use rustc_infer::traits::Obligation;
use rustc_middle::ty::{self, GenericArgs, TyCtxt, TypeVisitableExt, TypingMode, Unnormalized};
use rustc_span::DUMMY_SP;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::query::{resolve_fixed_projections, unknowns_to_placeholders, written_order};

/// The instantiated `where`-clause trait obligations of the impl that would satisfy `obligation`
/// (its direct dependencies), or `None` when no impl matches at all (so the caller can treat the
/// obligation as a terminal leaf). Found by unifying `obligation` with each candidate impl's trait
/// ref ([`match_impl`]), then instantiating and normalizing that impl's predicates. `Some(vec![])`
/// means an impl matched but carries no trait-clause `where` obligations.
///
/// A **concrete-`Self`** impl (one whose declared `Self` is a struct/enum, like the
/// `#[cgp_provider]` impl `impl ValueDeserializer<…> for DeserializeRecordFields`) is preferred
/// over a **blanket** one (whose `Self` is a bare type parameter, like the CGP delegation blanket
/// `impl<P: DelegateComponent> ValueDeserializer<…> for P`). Both unify with a provider obligation
/// such as `DeserializeRecordFields: ValueDeserializer<…>`, but only the specific impl's
/// `where`-clauses lead to the real cause; the blanket's lead to a
/// `DeserializeRecordFields: DelegateComponent` dead-end, since a leaf provider does not delegate.
/// A blanket impl is used only when no concrete-`Self` one matches, the usual case for an
/// obligation whose `Self` *is* the context (`App: CanUseComponent<…>` has only the blanket).
pub fn impl_where_obligations<'tcx>(
    tcx: TyCtxt<'tcx>,
    obligation: ty::PolyTraitClause<'tcx>,
) -> Option<Vec<ty::PolyTraitClause<'tcx>>> {
    let param_env = ty::ParamEnv::empty();

    for impl_did in impls_concrete_first(tcx, obligation.def_id()) {
        let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
        let ocx = ObligationCtxt::new(&infcx);
        let Some(impl_args) = match_impl(&ocx, obligation.map_bound(|p| p.trait_ref), impl_did)
        else {
            continue;
        };

        // Read the clauses in written order, so the causes they lead to are reported in it.
        let raw = written_order_clauses(tcx, impl_did, impl_args);
        // Register every predicate so the solver can propagate the constraints that *do* hold onto
        // the impl's otherwise-free parameters, before any single one is read. A record
        // deserializer's `Record: HasOptionalBuilder<Builder = Builder>` clause pins the free
        // `Builder` param to the concrete builder type; without solving it first, the sibling
        // `Record::Fields: HandleMapEntry<.., Builder>` clause (the branch that leads to the real
        // cause) carries `Builder` as a stray inference var and is dropped as inference-laden
        // below.
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
            if let Some(obligation) = read_where_clause(&ocx, clause) {
                obligations.push(obligation);
            }
        }

        return Some(obligations);
    }
    None
}

/// One `where` clause of a matched impl, as the walk reads it, or `None` when it is not a trait
/// clause.
///
/// The steps serve a later pipeline stage keyed on an earlier stage's output while the walk's input
/// `In` is a call-site unknown, a rigid placeholder, as in
/// `ProviderB: Handler<Ctx, Code, <ProviderA as Handler<Ctx, Code, In>>::Output>`. `ProviderA`'s
/// own `where` clause (`In: Send`) is false against the placeholder, so its impl is rejected and
/// the projection does not reduce, even when its `type Output` does not depend on `In`. In body
/// order:
///
/// - The parent's unifications, the obligation's placeholder input among them, are substituted
///   without normalizing, so the stalled projection survives as an alias; normalizing first would
///   collapse it into an opaque inference variable.
/// - [`resolve_fixed_projections`] resolves each such alias with the placeholders as deferrable
///   inference variables, so the blocking bound is ambiguous rather than false and the solver
///   commits to the sole impl and reads off its fixed output; the next stage's input is then the
///   concrete type that carries the real cause rather than an unknowable `_`.
/// - The clause is normalized.
/// - The inference variables left after solving, a parameter the impl match left unconstrained,
///   become rigid placeholders, so the walk still descends the stage instead of hiding every cause
///   past the first; the placeholder-leaf filter in
///   [`resolve_leaves`](crate::resolve::walk::resolve_leaves) keeps a leaf that depends on the
///   unknown from being reported, as for the call-site seed's unknown input.
///
/// The recovery and the placeholder steps change nothing in a clause with neither placeholders nor
/// stray inference variables, the common concrete-input case.
fn read_where_clause<'tcx>(
    ocx: &ObligationCtxt<'_, 'tcx>,
    clause: Unnormalized<'tcx, ty::Clause<'tcx>>,
) -> Option<ty::PolyTraitClause<'tcx>> {
    let infcx = ocx.infcx;
    let tcx = infcx.tcx;
    let pre = resolve_fixed_projections(
        tcx,
        infcx.deeply_resolve_ignoring_regions(clause.skip_norm_wip()),
    );
    let clause: ty::Clause<'tcx> = ocx.normalize(
        &ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        Unnormalized::new_wip(pre),
    );
    let clause = infcx.deeply_resolve_ignoring_regions(clause);
    let clause = if clause.has_non_region_infer() {
        unknowns_to_placeholders(tcx, clause)
    } else {
        clause
    };
    tcx.erase_and_anonymize_regions(clause).as_trait_clause()
}

/// The arguments of the impl `impl_did` when its trait ref unifies with `obligation` in `ocx`'s
/// inference context, or `None` when it does not. The impl's parameters start as fresh variables
/// ([`fresh_impl_ref`]) and its trait ref is normalized before it is related, so two impls whose
/// trait refs differ only in a projected argument are told apart: `eq` alone would leave the
/// projection as a goal and match both. This is the impl match the next solver allows, since
/// `SelectionContext` asserts against it, and each caller reads the matched impl's clauses its own
/// way.
///
/// The obligation's binder is instantiated with placeholders in this inference context first. A
/// higher-ranked obligation (`Self: for<'a> CanSerializeValue<&'a Value>`, the shape a recursive
/// provider like `SerializeIterator` carries) would otherwise reach `ocx.eq` through
/// `skip_binder()` with the `'a` bound var still escaping, tripping the inference generalizer's
/// `!source_term.has_escaping_bound_vars()` assertion and panicking rustc. Placeholders (rigid,
/// universal regions) let a *nested* higher-ranked hop resolve where fresh inference vars would
/// not: a projection through the bound lifetime (`<&'a Value as IntoIterator>::Item`) normalizes
/// deterministically against a placeholder region but stalls against an unconstrained inference
/// region. The fast path makes this a no-op for a binder-free obligation. The binder is
/// entered before the impl's variables are made, so they live in the placeholders' universe.
pub fn match_impl<'tcx>(
    ocx: &ObligationCtxt<'_, 'tcx>,
    obligation: ty::PolyTraitRef<'tcx>,
    impl_did: DefId,
) -> Option<GenericArgs<'tcx>> {
    let param_env = ty::ParamEnv::empty();
    let obligation_ref = ocx.infcx.enter_forall_and_leak_universe(obligation);
    let (impl_args, impl_ref) = fresh_impl_ref(ocx.infcx, impl_did);
    let impl_ref = ocx.normalize(
        &ObligationCause::dummy(),
        param_env,
        Unnormalized::new_wip(impl_ref),
    );
    ocx.eq(
        &ObligationCause::dummy(),
        param_env,
        obligation_ref,
        impl_ref,
    )
    .ok()?;
    Some(impl_args)
}

/// Fresh variables of `infcx` for the impl `impl_did`'s parameters, and its trait ref instantiated
/// with them, unnormalized.
pub fn fresh_impl_ref<'tcx>(
    infcx: &InferCtxt<'tcx>,
    impl_did: DefId,
) -> (GenericArgs<'tcx>, ty::TraitRef<'tcx>) {
    let tcx = infcx.tcx;
    let impl_args = infcx.fresh_args_for_item(DUMMY_SP, impl_did);
    let impl_ref = tcx
        .impl_trait_ref(impl_did)
        .instantiate(tcx, impl_args)
        .skip_norm_wip();
    (impl_args, impl_ref)
}

/// The clauses of `impl_did` instantiated with `args`, in the order the program wrote them
/// ([`written_order`](fn@written_order)), or in the compiler's order when the instantiation does
/// not keep one clause per written clause.
pub fn written_order_clauses<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_did: DefId,
    args: GenericArgs<'tcx>,
) -> Vec<Unnormalized<'tcx, ty::Clause<'tcx>>> {
    let generic = tcx.clauses_of(impl_did);
    let instantiated: Vec<Unnormalized<'tcx, ty::Clause<'tcx>>> = generic
        .instantiate(tcx, args)
        .into_iter()
        .map(|(clause, _)| clause)
        .collect();
    if instantiated.len() == generic.clauses.len() {
        written_order(tcx, generic.clauses)
            .into_iter()
            .map(|index| instantiated[index])
            .collect()
    } else {
        instantiated
    }
}

/// Every impl of `trait_did`, with the concrete-`Self` impls ahead of the blanket (param-`Self`)
/// ones, so a caller that takes the first *unifying* impl prefers the specific impl over the
/// delegation blanket. [`impl_where_obligations`],
/// [`projection_mismatch`](fn@super::projection_mismatch), and the classifier's
/// `required_field_type` rely on this order.
pub fn impls_concrete_first(tcx: TyCtxt<'_>, trait_did: DefId) -> Vec<DefId> {
    let (blanket, concrete): (Vec<DefId>, Vec<DefId>) =
        tcx.all_impls(trait_did).partition(|&did| {
            matches!(
                tcx.impl_trait_ref(did).skip_binder().self_ty().kind(),
                ty::Param(_)
            )
        });
    concrete.into_iter().chain(blanket).collect()
}
