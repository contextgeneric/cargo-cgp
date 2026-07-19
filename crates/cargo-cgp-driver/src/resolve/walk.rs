//! Walking the wiring's dependency graph down to each terminal root cause.
//!
//! From a starting obligation (recovered by [anchor](crate::resolve::anchor)) this descends the
//! failing trait obligations — following only the CGP wiring vocabulary and obligations on the
//! context itself — and collects every root→leaf path that bottoms out on a terminal unmet bound,
//! folding each into a [`Cause`] with its rendered dependency tree.

use cargo_cgp_error_processing::{Cause, Resolved, dependency_tree_leaf};
use rustc_infer::infer::{InferCtxt, TyCtxtInferExt};
use rustc_infer::traits::Obligation;
use rustc_middle::ty::{
    self, Ty, TyCtxt, TypeFoldable, TypeFolder, TypeSuperFoldable, TypeVisitableExt, TypingMode,
    Unnormalized, Upcast as _,
};
use rustc_span::DUMMY_SP;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE, DELEGATE_COMPONENT_TRAIT,
    HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
};
use crate::resolve::cgp_item::{is_cgp_item, is_provider_trait};
use crate::resolve::classify::{classify_leaf, is_reportable_leaf};
use crate::resolve::label::{label_for, spine, trait_generics};

/// Bound on how deep the dependency-graph walk descends before giving up, so a pathological or
/// cyclic wiring cannot make it loop. Each logical wiring hop expands to several walk frames — a
/// consumer obligation, the delegation-routing provider obligation, a `RedirectLookup`, the real
/// provider obligation, then the next consumer — so a deeply nested data type (e.g. `cgp-serde`'s
/// `MessagesArchive`, a `Vec<Vec<record>>` threaded through iterator/deref/record providers) reaches
/// its root cause only tens of frames down. The bound is set well above that so a genuine chain
/// resolves rather than declining to the raw fallback, but not so high that a *divergent* wiring —
/// one whose obligations keep growing without ever exactly repeating, which the cycle guard cannot
/// catch — grinds through the trait solver at every frame for a long time before giving up.
const MAX_DEPTH: u32 = 256;

/// Walk the dependency graph of `top` — the **real consumer-trait obligation** `Ctx:
/// ConsumerTrait<Params…>` the failure stands for, not a `CanUseComponent` wrapper — and, for each
/// distinct terminal unmet bound it bottoms out on, return that leaf with its rendered dependency
/// chain. The walk descends the consumer trait to its provider trait and on to the provider's real
/// `where` bounds, never through `IsProviderFor`. `None` when no branch reaches a resolvable leaf.
pub(crate) fn resolve_leaves<'tcx>(
    tcx: TyCtxt<'tcx>,
    top: ty::PolyTraitPredicate<'tcx>,
) -> Option<Resolved> {
    // Erase the seed's free regions up front, exactly as every descendant obligation is erased by
    // [`impl_where_obligations`]. A lifetime-parameterized check entry (`(Life<'a>, str)`) seeds an
    // obligation carrying the check impl's own `'a`, and leaving it unerased would make the seed the
    // one chain entry whose `self_ty == context` comparisons fail, mislabeling the consumer node.
    let top = tcx.erase_and_anonymize_regions(top);
    let context = top.skip_binder().self_ty();

    let mut causes: Vec<Cause> = Vec::new();
    for path in collect_leaf_paths(tcx, top, context, &[], 0) {
        // Split off the terminal (leaf) predicate: the chain above it becomes the tree's inner
        // nodes, and the leaf itself is re-stated as the tree's final leaf below, so the chain
        // always bottoms out at the root cause rather than one step before it.
        let Some((leaf_pred, chain)) = path.preds.split_last() else {
            continue;
        };
        let leaf_ref = leaf_pred.skip_binder().trait_ref;
        // A leaf still carrying one of the seed's unknown-parameter placeholders (the call-site
        // anchor's stand-in for a call's inferred input) is a bound on a type the recovery could
        // not know; reporting it would fabricate a requirement (`_: Send`) the programmer cannot
        // act on, so only a placeholder-free leaf — a dependency that fails whatever the unknown
        // parameter is — is kept. Placeholder *regions* never reach here: a higher-ranked hop's
        // leaked placeholders are erased with the rest of a child's regions.
        if leaf_pred.has_placeholders() {
            continue;
        }
        // A path that bottoms out on pure wiring plumbing (a routing dead-end) is not a root
        // cause — a real cause is found down another branch — so drop it rather than report it.
        // The obligation one hop above the leaf (its parent in the chain) is the impl whose
        // `where`-clause produced the leaf; its `Self` tells a `DelegateComponent` dispatch lookup
        // into a separate table (`Components` inside `UseDelegate<Components>`) from the self-keyed
        // delegation blanket, and its trait is the provider trait a non-provider leaf names against.
        let parent = chain.last().map(|parent| parent.skip_binder().trait_ref);
        if !is_reportable_leaf(tcx, leaf_ref, context, parent) {
            continue;
        }
        let leaf = classify_leaf(tcx, leaf_ref, context, parent, path.mismatch);
        // One sub-error per distinct leaf: a leaf wanted by several branches is one fix.
        if causes.iter().any(|c| c.key() == leaf.key()) {
            continue;
        }
        let mut labels: Vec<String> = chain
            .iter()
            .filter_map(|pred| label_for(tcx, *pred, context))
            .collect();
        // Repeat the root cause as the terminal leaf node, so the tree ends on it — the same shape
        // whether the leaf is a missing field, an unmet bound, a missing wiring, or a redirect. As
        // a tree entry it carries its own `CGP-E1xx` code (except a pass-through non-CGP bound).
        labels.push(dependency_tree_leaf(&leaf));
        if let Some(tree) = spine(labels) {
            causes.push(Cause { leaf, tree });
        }
    }

    if causes.is_empty() {
        return None;
    }

    // The consumer trait is the seed obligation's own trait, named straight off its `DefId` with
    // its parameters reattached from the obligation's arguments — no marker, no name map, no
    // `IsProviderFor`. The `DefId` is exact, so two same-named components in different modules
    // cannot be confused.
    let top_ref = top.skip_binder().trait_ref;
    let consumer = format!(
        "{}{}",
        tcx.item_name(top_ref.def_id),
        trait_generics(tcx, top_ref, 1)
    );

    Some(Resolved {
        context: context.to_string(),
        consumers: vec![consumer],
        // Every anchor seeds the walk with a real CGP consumer obligation (the impl-site anchor
        // overrides this when the failing trait is a plain wrapper).
        consumers_are_cgp: true,
        // The subject is the checked context itself.
        subject_is_context: true,
        causes,
    })
}

/// One collected root→leaf path: the chain of trait predicates from the top down to (and
/// including) the leaf, plus — when the leaf is a field-type mismatch — the type the failing
/// projection required. A `mismatch` of `Some(expected)` means the last predicate is a `HasField`
/// bound that *holds* as a trait, and the real fault is the associated-type projection `Value ==
/// expected`; `None` means an ordinary unmet-bound leaf.
struct LeafPath<'tcx> {
    preds: Vec<ty::PolyTraitPredicate<'tcx>>,
    mismatch: Option<Ty<'tcx>>,
}

/// Collect every root→leaf path that bottoms out on a terminal unmet bound, by descending the
/// failing obligation's dependency graph. `pred` is a failing obligation and `prefix` is the path
/// of predicates above it. A `HasField` completes a path directly; any other obligation contributes
/// the `where`-clause obligations of the impl that would satisfy it, recursing into just the ones
/// that do not already hold. An obligation with **no** satisfying impl is itself a terminal leaf
/// (an ordinary bound like `f64: Eq`). Following *every* unmet dependency (not one) is what surfaces
/// independent causes as separate paths. Bounded by [`MAX_DEPTH`].
///
/// One case does not descend by trait clauses: an unmet obligation whose satisfying impl's
/// trait-clause `where`-obligations all *hold*. The obligation is then unmet for a reason the
/// trait-clause walk cannot see — a projection/associated-type mismatch. The resolver looks for one
/// specific, reportable form: a `HasField` projection (`<Ctx as HasField<Symbol!(..)>>::Value ==
/// T`) among the impl's own predicates that does not hold — a field present with the wrong type. It
/// completes the path with that field's `HasField` trait ref and records the expected type as the
/// path's `mismatch`, so the caller renders a [`FieldTypeMismatch`](cargo_cgp_error_processing::Leaf::FieldTypeMismatch).
/// A branch with no such projection yields nothing, so the resolver declines it to the fallback.
///
/// A bound on a foreign type (whose `Self` is not the context and whose trait is not CGP wiring)
/// is normally the terminal leaf — that bound *is* the root cause a reader wants (`f64: Eq`), and
/// descending it blindly would wander into whatever unrelated `std` blanket impl happens to match
/// its `Self` (e.g. `impl<F: FnPtr> Eq for F`) and fabricate a misleading chain. Two foreign bounds
/// are the exception. One is satisfied by an impl that itself depends on the *context* — a request
/// struct's `HasBasicAuthHeader<Ctx>` getter, whose `#[cgp_auto_getter]` blanket impl requires
/// `Ctx: HasPasswordType`: there the descent follows only the impl's context-side dependencies (so
/// the real cause on the context surfaces and de-duplicates with the same cause reached elsewhere).
/// The other is a **same-trait recursion over a type-level list** — a record's `Cons<Field<..>, ..>:
/// HandleMapEntry<..>` whose tail is another `Cons<.., Nil>: HandleMapEntry<..>` — which the descent
/// follows into so a field deep in the list whose value type is unwired is still reached. Following
/// only context-side deps *and* the same trait keeps the `f64: Eq` guarantee intact: a foreign
/// `f64: FnPtr` step is neither, so it is never followed and the bound stays the leaf.
fn collect_leaf_paths<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
    prefix: &[ty::PolyTraitPredicate<'tcx>],
    depth: u32,
) -> Vec<LeafPath<'tcx>> {
    if depth > MAX_DEPTH {
        return Vec::new();
    }

    // Cycle guard: if this exact obligation already appears among its own ancestors, the wiring
    // loops (a `UseContext` cycle routes `Ctx: CanUseComponent<C>` straight back to itself), so this
    // branch carries no new root cause — stop rather than descend the loop. This is what lets
    // [`MAX_DEPTH`] be a high backstop for genuinely deep chains without a cycle spinning down to it
    // (and overflowing the recursion's stack): a real cycle bottoms out here, at its first repeat,
    // not at the depth cap. Regions are erased so a loop that only differs by lifetime is still seen.
    let erased = tcx.erase_and_anonymize_regions(pred);
    if prefix
        .iter()
        .any(|ancestor| tcx.erase_and_anonymize_regions(*ancestor) == erased)
    {
        return Vec::new();
    }

    let mut path = prefix.to_vec();
    path.push(pred);

    if is_has_field(tcx, pred) {
        return vec![LeafPath {
            preds: path,
            mismatch: None,
        }];
    }

    let descendable = is_descendable(tcx, pred, context);

    let Some(children) = impl_where_obligations(tcx, pred) else {
        // No impl satisfies `pred` at all — `pred` is itself the terminal root-cause bound.
        return vec![LeafPath {
            preds: path,
            mismatch: None,
        }];
    };

    let unmet: Vec<_> = children
        .into_iter()
        .filter(|nested| !holds(tcx, *nested))
        // Drop the check-trait scaffolding wherever it appears as a dependency: the generated
        // blanket impls carry a `CanUseComponent`/`IsProviderFor` bound *beside* the real consumer
        // or provider-trait obligation, and only the latter is walked — the `IsProviderFor` bound
        // just re-states the provider's `where` clause, which the real provider impl already
        // carries. Following it would route the cause through `IsProviderFor` (and let its copy of
        // the bounds win the per-leaf de-duplication over the real provider chain), which is exactly
        // the dependency on `IsProviderFor` cargo-cgp is shedding.
        .filter(|nested| !is_workaround_plumbing(tcx, *nested))
        .collect();

    if !descendable {
        // A foreign-type bound — its `Self` is not the context and its trait is not CGP wiring —
        // is normally the terminal root cause, and the descent must not walk into whatever `std`
        // blanket impl happens to satisfy it (an `impl<F: FnPtr> Eq for F` would fabricate a
        // misleading `f64: FnPtr` step). But a CGP getter or capability trait applied to a
        // *non-context* type — a request struct's `HasBasicAuthHeader<Ctx>`, whose
        // `#[cgp_auto_getter]` blanket impl requires `Ctx: HasPasswordType` — is often unmet only
        // because a dependency *on the context* is unmet. So look into that blanket impl and
        // descend into just its context-side dependencies, which reveals the real cause (and lets
        // it de-duplicate with the same cause reached down another branch). The context-side
        // filter is what keeps the `f64: Eq` guarantee: a foreign `f64: FnPtr` step is not
        // context-side, so it is never followed and the bound stays the leaf. It also skips the
        // getter's own `Ctx::Assoc`-typed `HasField` clause on the request (present, but a
        // projection mismatch), which a plain descent would misreport as a missing field.
        //
        // A foreign trait that recurses over a type-level list also reaches the context only
        // deeper: a record's `Cons<Field<.., V0>, Cons<Field<.., V1>, Nil>>: HandleMapEntry<.., Ctx,
        // ..>` handles its head field's `Ctx: CanDeserializeValue<V0>` here but its later fields
        // through the **tail** `Cons<.., Nil>: HandleMapEntry<..>` — a same-trait bound on another
        // foreign list node. So a same-trait recursion is followed alongside the context-side deps,
        // which lets the walk reach the field whose dependency is the real cause. Following only the
        // *same* trait keeps the `f64: Eq` guarantee: a foreign leaf's `impl` dep is a *different*
        // trait (`f64: FnPtr`), so it is never mistaken for a structural recursion.
        let this_trait = pred.def_id();
        let followable: Vec<_> = unmet
            .into_iter()
            .filter(|nested| is_descendable(tcx, *nested, context) || nested.def_id() == this_trait)
            .collect();
        if followable.is_empty() {
            return vec![LeafPath {
                preds: path,
                mismatch: None,
            }];
        }
        let mut paths = Vec::new();
        for nested in followable {
            paths.extend(collect_leaf_paths(tcx, nested, context, &path, depth + 1));
        }
        return paths;
    }

    if unmet.is_empty() {
        // Matched an impl, yet every trait-clause `where`-obligation holds: the fault is a
        // projection the trait-clause walk cannot see. Surface the one form we can pin down — a
        // `HasField::Value` mismatch (a field present with the wrong type) — and decline anything
        // else to the fallback.
        return match has_field_projection_mismatch(tcx, pred) {
            Some((field_ref, expected)) => {
                path.push(ty::Binder::dummy(field_ref).upcast(tcx));
                vec![LeafPath {
                    preds: path,
                    mismatch: Some(expected),
                }]
            }
            None => Vec::new(),
        };
    }

    let mut paths = Vec::new();
    for nested in unmet {
        paths.extend(collect_leaf_paths(tcx, nested, context, &path, depth + 1));
    }
    paths
}

/// When the impl that satisfies `pred`'s trait obligation carries an unmet `HasField`
/// associated-type projection — `<Ctx as HasField<Symbol!("height")>>::Value == f64` — return that
/// field's `HasField` trait ref (the terminal the tree shows) paired with the expected type
/// (`f64`). This is the field-present-with-wrong-type case: the trait bound holds, so the walk
/// reaches it only here, in the branch where every trait-clause dependency held. `None` when the
/// impl carries no such unmet `HasField` projection.
///
/// Mirrors [`impl_where_obligations`]'s next-solver-safe impl match (`fresh_args_for_item` + `eq`),
/// but keeps the projection predicates rather than the trait ones, and leaves each projection
/// un-normalized so its `<.. as HasField<..>>::Value` alias survives for the hold check.
fn has_field_projection_mismatch<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
) -> Option<(ty::TraitRef<'tcx>, Ty<'tcx>)> {
    // Prefer a concrete-`Self` impl, falling back to a blanket, exactly as
    // [`impl_where_obligations`] does. For a provider trait both the concrete provider impl (which
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
    // as [`impl_where_obligations`]: a `skip_binder()`'d escaping bound var fed into `ocx.eq` panics
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

/// Whether `pred` is the check-trait scaffolding — `CanUseComponent` or `IsProviderFor` — the walk
/// resolves *around* rather than through. These sit beside the real consumer/provider-trait
/// obligation in every generated blanket impl and carry only the delegation check
/// (`CanUseComponent`) or a copy of the provider's `where` clause (`IsProviderFor`), both redundant
/// with the real obligation the walk follows instead. Dropping them as dependencies is what keeps
/// the resolution independent of `IsProviderFor`.
fn is_workaround_plumbing<'tcx>(tcx: TyCtxt<'tcx>, pred: ty::PolyTraitPredicate<'tcx>) -> bool {
    let did = pred.skip_binder().trait_ref.def_id;
    is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
}

/// Whether the descent should walk *into* `pred`'s dependencies, rather than treat `pred` as a
/// terminal leaf. It descends any **provider trait** (a `ProvideFoo: Foo<App>` bound routes on to
/// the provider's own real `where` bounds), the `DelegateComponent` table lookup, and any
/// obligation on the context itself (its consumer, getter, and capability traits). It stops at
/// everything else — an ordinary bound like `f64: Eq`, whose `Self` is a foreign type, is a leaf.
///
/// It deliberately does *not* descend `CanUseComponent`/`IsProviderFor`: the resolver reads a
/// provider's bounds from the provider trait's own impl, not from the `IsProviderFor` marker's
/// copy of them, so those obligations are dropped as [workaround plumbing](is_workaround_plumbing)
/// while the real provider-trait obligation beside them carries the cause.
fn is_descendable<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
) -> bool {
    let trait_ref = pred.skip_binder().trait_ref;
    let did = trait_ref.def_id;
    tcx.erase_and_anonymize_regions(trait_ref.self_ty()) == context
        || is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_provider_trait(tcx, did)
}

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
/// to the real cause; the blanket's lead to a `DeserializeRecordFields: DelegateComponent` dead-end,
/// since a leaf provider does not delegate. A blanket impl is used only when no concrete-`Self` one
/// matches — the usual case for an obligation whose `Self` *is* the context (`App: CanUseComponent<…>`
/// has only the blanket).
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
            // [`resolve_leaves`] keeps any leaf that genuinely depends on the unknown from being
            // reported, exactly as for the call-site seed's unknown input. A no-op when there are
            // no such vars (the common concrete-input case), so ordinary walks are unaffected.
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

/// Replace every unresolved inference variable in `value` with a rigid placeholder, so it can
/// cross inference-context boundaries (a placeholder is a rigid type constant, not tied to an
/// `InferCtxt`). Distinct variables get distinct placeholders and repeated occurrences of one
/// variable the same one (keyed by the variable's index, the integer and float spaces offset
/// apart), preserving whatever type equalities unification established. Used both to seed the
/// [call-site anchor](crate::resolve::call_site)'s unknown call arguments and to keep a
/// later-pipeline-stage clause walkable in [`impl_where_obligations`].
pub(crate) fn unknowns_to_placeholders<'tcx, T>(tcx: TyCtxt<'tcx>, value: T) -> T
where
    T: TypeFoldable<TyCtxt<'tcx>>,
{
    struct Folder<'tcx> {
        tcx: TyCtxt<'tcx>,
    }
    impl<'tcx> TypeFolder<TyCtxt<'tcx>> for Folder<'tcx> {
        fn cx(&self) -> TyCtxt<'tcx> {
            self.tcx
        }
        fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
            let var = match *ty.kind() {
                ty::Infer(ty::TyVar(vid)) => vid.as_u32(),
                ty::Infer(ty::IntVar(vid)) => (1 << 20) + vid.as_u32(),
                ty::Infer(ty::FloatVar(vid)) => (1 << 21) + vid.as_u32(),
                ty::Infer(_) => 1 << 22,
                _ if !ty.has_infer() => return ty,
                _ => return ty.super_fold_with(self),
            };
            Ty::new_placeholder(
                self.tcx,
                ty::PlaceholderType::new_anon(ty::UniverseIndex::ROOT, ty::BoundVar::from_u32(var)),
            )
        }
    }
    value.fold_with(&mut Folder { tcx })
}

/// Replace any **fixed** associated-type projection in `clause` that stalls on an unknown-input
/// placeholder with the concrete type it reduces to. See the call site in
/// [`impl_where_obligations`] for why this matters: a later pipeline stage's input is an earlier
/// stage's `::Output`, and against a rigid placeholder input the earlier provider's `where` clause
/// is *false* (not merely unproven), so the projection never reduces even though the provider's
/// `type Output` is independent of the input. A no-op unless the clause carries a placeholder.
fn resolve_fixed_projections<'tcx>(
    tcx: TyCtxt<'tcx>,
    clause: ty::Clause<'tcx>,
) -> ty::Clause<'tcx> {
    if !clause.has_placeholders() {
        return clause;
    }
    clause.fold_with(&mut ProjectionResolver { tcx })
}

/// Folds a value, replacing each associated-type projection that both carries an unknown-input
/// placeholder and reduces to a concrete type (via [`try_project_fixed`]) with that concrete type.
struct ProjectionResolver<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> TypeFolder<TyCtxt<'tcx>> for ProjectionResolver<'tcx> {
    fn cx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        // Only a projection that mentions an unknown can be one that stalled on the unknown input;
        // a placeholder-free subtree is left untouched (and its recursion pruned).
        if !ty.has_placeholders() {
            return ty;
        }
        if let ty::Alias(_, alias) = ty.kind()
            && matches!(alias.kind, ty::AliasTyKind::Projection { .. })
            && let Some(concrete) = try_project_fixed(self.tcx, ty)
        {
            return concrete;
        }
        ty.super_fold_with(self)
    }
}

/// Try to reduce one associated-type projection `alias_ty` to a concrete type, treating the
/// unknown-input placeholders inside it as *deferrable* inference variables rather than rigid
/// falsifiers. Returns the reduced type only when it is fully concrete — no inference variable or
/// placeholder left (so it does not depend on the unknown input) and no longer an alias (the
/// projection genuinely reduced). `None` otherwise, leaving the projection as it was.
///
/// This is the one extra trait-solver interaction the walk performs here, and it is the safe kind:
/// a fresh `InferCtxt` normalizing a concrete goal, forcing only the cached queries type-checking
/// already ran (see the panic hazards in `docs/implementation/rustc-diagnostic-internals.md`).
fn try_project_fixed<'tcx>(tcx: TyCtxt<'tcx>, alias_ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    let goal = placeholders_to_infer(&infcx, alias_ty);
    let normalized = ocx.normalize(
        &ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        Unnormalized::new_wip(goal),
    );
    let _ = ocx.try_evaluate_obligations();
    let normalized = tcx.erase_and_anonymize_regions(infcx.resolve_vars_if_possible(normalized));
    if normalized.has_non_region_infer()
        || normalized.has_placeholders()
        || matches!(normalized.kind(), ty::Alias(..))
    {
        return None;
    }
    Some(normalized)
}

/// Replace every unknown-input placeholder (a root-universe anonymous placeholder, as minted by
/// [`unknowns_to_placeholders`]) with a fresh inference variable, one per distinct placeholder so
/// equalities are preserved. A leaked higher-ranked binder's placeholder (a non-root universe) is
/// left alone. The inverse of [`unknowns_to_placeholders`], used to let the solver *defer* a bound
/// on the unknown rather than reject it while a fixed projection is reduced.
fn placeholders_to_infer<'tcx, T>(infcx: &InferCtxt<'tcx>, value: T) -> T
where
    T: TypeFoldable<TyCtxt<'tcx>>,
{
    struct Folder<'a, 'tcx> {
        infcx: &'a InferCtxt<'tcx>,
        vars: std::collections::HashMap<u32, Ty<'tcx>>,
    }
    impl<'tcx> TypeFolder<TyCtxt<'tcx>> for Folder<'_, 'tcx> {
        fn cx(&self) -> TyCtxt<'tcx> {
            self.infcx.tcx
        }
        fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
            if let ty::Placeholder(p) = *ty.kind()
                && p.universe == ty::UniverseIndex::ROOT
            {
                let key = p.bound.var.as_u32();
                return *self
                    .vars
                    .entry(key)
                    .or_insert_with(|| self.infcx.next_ty_var(DUMMY_SP));
            }
            if !ty.has_placeholders() {
                return ty;
            }
            ty.super_fold_with(self)
        }
    }
    value.fold_with(&mut Folder {
        infcx,
        vars: std::collections::HashMap::new(),
    })
}

/// Every impl of `trait_did`, with the concrete-`Self` impls ahead of the blanket (param-`Self`)
/// ones, so a caller that takes the first *unifying* impl prefers the specific impl over the
/// delegation blanket — the ordering [`impl_where_obligations`] and
/// [`has_field_projection_mismatch`] both rely on.
fn impls_concrete_first(tcx: TyCtxt<'_>, trait_did: DefId) -> Vec<DefId> {
    let (blanket, concrete): (Vec<DefId>, Vec<DefId>) =
        tcx.all_impls(trait_did).partition(|&did| {
            matches!(
                tcx.impl_trait_ref(did).skip_binder().self_ty().kind(),
                ty::Param(_)
            )
        });
    concrete.into_iter().chain(blanket).collect()
}

/// Whether `pred` already holds — a dependency that is satisfied and so is not descended into.
pub(crate) fn holds<'tcx>(tcx: TyCtxt<'tcx>, pred: ty::PolyTraitPredicate<'tcx>) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), ty::ParamEnv::empty(), pred);
    infcx.predicate_must_hold_modulo_regions(&obligation)
}

/// Whether an associated-type projection already holds — used to tell a matching field type from a
/// mismatched one (`<Rectangle as HasField<Symbol!("height")>>::Value == f64` holds when `height`
/// is `f64`, fails when it is `i32`).
fn holds_projection<'tcx>(tcx: TyCtxt<'tcx>, pred: ty::PolyProjectionPredicate<'tcx>) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), ty::ParamEnv::empty(), pred);
    infcx.predicate_must_hold_modulo_regions(&obligation)
}

/// Whether a trait predicate is a genuine CGP `HasField` bound — the missing-field leaf.
fn is_has_field(tcx: TyCtxt<'_>, pred: ty::PolyTraitPredicate<'_>) -> bool {
    is_cgp_item(
        tcx,
        pred.skip_binder().def_id(),
        HAS_FIELD_TRAIT,
        CGP_FIELD_CRATE,
    )
}
