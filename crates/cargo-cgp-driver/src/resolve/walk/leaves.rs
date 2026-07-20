//! Descending the failing obligation to every terminal root-cause leaf.

use cargo_cgp_error_processing::tree::DependencyTree;
use cargo_cgp_error_processing::{Cause, Resolved, dependency_tree_leaf, elide_repeated_generics};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt, Upcast as _};

use crate::resolve::cache::{NodeKey, ResolveCache};
use crate::resolve::classify::{classify_leaf, is_reportable_leaf};
use crate::resolve::label::{label_for, trait_generics};
use crate::resolve::walk::{
    has_field_projection_mismatch, holds, impl_where_obligations, is_descendable, is_has_field,
    is_workaround_plumbing,
};

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
///
/// Memoized on the region-erased seed and its context through `cache`: one CGP mistake surfaces the
/// same failure at many sites, each seeding this walk with the same obligation, so the walk runs
/// once and its owned result is reused. See `docs/implementation/cached-dependency-resolution.md`.
pub(crate) fn resolve_leaves<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    top: ty::PolyTraitPredicate<'tcx>,
) -> Option<Resolved> {
    // Erase the seed's free regions up front, exactly as every descendant obligation is erased by
    // [`impl_where_obligations`]. A lifetime-parameterized check entry (`(Life<'a>, str)`) seeds an
    // obligation carrying the check impl's own `'a`, and leaving it unerased would make the seed the
    // one chain entry whose `self_ty == context` comparisons fail, mislabeling the consumer node.
    let top = tcx.erase_and_anonymize_regions(top);
    let context = top.skip_binder().self_ty();

    // The context is part of the key because the rendering compares node self-types against it. The
    // borrow is released before `compute_leaves` runs, so the memo never holds it across the compute.
    let key = NodeKey::new(tcx, top, context);
    if let Some(hit) = cache.get(&key) {
        return hit;
    }
    let result = compute_leaves(tcx, top, context);
    cache.insert(key, result.clone());
    result
}

/// The uncached core of [`resolve_leaves`]: walk the already region-erased seed `top` under root
/// `context` and fold each root→leaf path into a [`Cause`].
fn compute_leaves<'tcx>(
    tcx: TyCtxt<'tcx>,
    top: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
) -> Option<Resolved> {
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
        // A dispatch chain's plumbing hops all restate the same program-sized trait parameters;
        // elide a hop that exactly repeats its predecessor so the chain reads as its steps.
        let labels = elide_repeated_generics(labels);
        if let Some(tree) = DependencyTree::from_chain(labels) {
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
/// path's `mismatch`, so the caller renders a
/// [`FieldTypeMismatch`](cargo_cgp_error_processing::Leaf::FieldTypeMismatch). A branch with no
/// such projection yields nothing, so the resolver declines it to the fallback.
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
