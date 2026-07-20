//! Descending the failing obligation to every terminal root-cause leaf.

use cargo_cgp_error_processing::tree::DependencyTree;
use cargo_cgp_error_processing::{Cause, Resolved, dependency_tree_leaf, elide_repeated_generics};
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt, Upcast as _};

use crate::resolve::cache::{NodeKey, ResolveCache, SubCause, SubResult, pred_fingerprint};
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
/// The descent is memoized at **every node** through `cache`: one CGP mistake surfaces the same
/// failure at many sites (all seeding the same obligation), and a shared capability is a diamond
/// reached from several parents, so each distinct obligation is resolved once and reused. See
/// `docs/implementation/cached-dependency-resolution.md`.
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
    compute_leaves(tcx, cache, top, context)
}

/// Fold the root node's owned sub-causes into a [`Resolved`]: de-duplicate by leaf (one sub-error per
/// distinct root cause, first occurrence kept), elide repeated generics on each chain, and render the
/// tree. `top` is already region-erased and `context` is its self type.
fn compute_leaves<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    top: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
) -> Option<Resolved> {
    let sub = resolve_node(tcx, cache, top, None, context, &[], 0);

    let mut causes: Vec<Cause> = Vec::new();
    for sc in sub.causes {
        // One sub-error per distinct leaf: a leaf wanted by several branches is one fix.
        if causes.iter().any(|c| c.key() == sc.leaf.key()) {
            continue;
        }
        // A dispatch chain's plumbing hops all restate the same program-sized trait parameters;
        // elide a hop that exactly repeats its predecessor so the chain reads as its steps.
        let labels = elide_repeated_generics(sc.labels);
        if let Some(tree) = DependencyTree::from_chain(labels) {
            causes.push(Cause {
                leaf: sc.leaf,
                tree,
            });
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

/// Resolve one node of the dependency graph to its owned, node-rooted sub-causes, memoized in
/// `cache`. `pred` is the (already region-erased) failing obligation, `parent` the trait of the
/// obligation directly above it (needed only to classify a terminal leaf), `context` the root
/// context, and `prefix` the chain of ancestors above `pred` (for the cycle guard and the reuse
/// disjointness check). A `HasField`, or an obligation with no satisfying impl, or a foreign bound
/// with nothing context-side to follow, is a terminal leaf; anything else descends into the unmet
/// `where`-obligations of the impl that would satisfy it. Only complete (untainted) non-terminal
/// nodes are cached.
fn resolve_node<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    pred: ty::PolyTraitPredicate<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    context: Ty<'tcx>,
    prefix: &[ty::PolyTraitPredicate<'tcx>],
    depth: u32,
) -> SubResult {
    // Depth cap: a divergent wiring the cycle guard cannot catch (obligations that keep growing
    // without repeating) is cut here. The cut flags the subtree incomplete so it is never cached.
    if depth > MAX_DEPTH {
        return SubResult::cut();
    }

    // Cycle guard: if this exact obligation already appears among its own ancestors, the wiring
    // loops (a `UseContext` cycle routes `Ctx: CanUseComponent<C>` straight back to itself), so this
    // branch carries no new root cause — stop rather than descend the loop. Regions are erased so a
    // loop that only differs by lifetime is still seen. The cut flags the subtree incomplete, which
    // propagates up so no ancestor whose result depended on the cut is cached.
    let erased = tcx.erase_and_anonymize_regions(pred);
    if prefix
        .iter()
        .any(|ancestor| tcx.erase_and_anonymize_regions(*ancestor) == erased)
    {
        return SubResult::cut();
    }

    // Interior-cache consult. Only complete non-terminal nodes are ever stored, so this hits only a
    // fully-explored subtree; reuse it only when no current ancestor lies inside it, since otherwise
    // splicing it would keep a branch a fresh walk's cycle guard would cut here.
    let key = NodeKey::new(tcx, erased, context);
    if let Some(cached) = cache.get(&key) {
        let reusable = prefix.iter().all(|ancestor| {
            !cached.reachable.contains(&pred_fingerprint(
                tcx,
                tcx.erase_and_anonymize_regions(*ancestor),
            ))
        });
        if reusable {
            return cached;
        }
    }

    let self_fp = pred_fingerprint(tcx, erased);
    let leaf_ref = erased.skip_binder().trait_ref;

    // A `HasField` completes a path directly — a terminal leaf, not cached (its classification can
    // read its parent, which lies outside a leaf-rooted subtree, and it is cheap to re-derive).
    if is_has_field(tcx, erased) {
        return terminal_result(tcx, erased, leaf_ref, parent, context, self_fp);
    }

    let descendable = is_descendable(tcx, erased, context);

    let Some(children) = impl_where_obligations(tcx, erased) else {
        // No impl satisfies `pred` at all — `pred` is itself the terminal root-cause bound.
        return terminal_result(tcx, erased, leaf_ref, parent, context, self_fp);
    };

    let unmet: Vec<_> = children
        .into_iter()
        .filter(|nested| !holds(tcx, *nested))
        // Drop the check-trait scaffolding wherever it appears as a dependency: the generated
        // blanket impls carry a `CanUseComponent`/`IsProviderFor` bound *beside* the real consumer
        // or provider-trait obligation, and only the latter is walked — the `IsProviderFor` bound
        // just re-states the provider's `where` clause, which the real provider impl already carries.
        .filter(|nested| !is_workaround_plumbing(tcx, *nested))
        .collect();

    // Decide the children to descend, or return a terminal / projection result directly.
    let children: Vec<ty::PolyTraitPredicate<'tcx>> = if !descendable {
        // A foreign-type bound is normally the terminal root cause, and the descent must not wander
        // into whatever `std` blanket impl happens to satisfy it. Two exceptions are followed: a CGP
        // getter/capability on a non-context type whose blanket impl depends on the *context* (so the
        // real cause surfaces and de-duplicates), and a same-trait recursion over a type-level list
        // (a record's `Cons<..>: HandleMapEntry<..>` whose tail is another `Cons<.., Nil>: …`). A
        // foreign `f64: FnPtr` step is neither, so the bound stays the leaf.
        let this_trait = erased.def_id();
        let followable: Vec<_> = unmet
            .into_iter()
            .filter(|nested| is_descendable(tcx, *nested, context) || nested.def_id() == this_trait)
            .collect();
        if followable.is_empty() {
            return terminal_result(tcx, erased, leaf_ref, parent, context, self_fp);
        }
        followable
    } else if unmet.is_empty() {
        // Matched an impl, yet every trait-clause `where`-obligation holds: the fault is a projection
        // the trait-clause walk cannot see. Surface the one form we can pin down — a `HasField::Value`
        // mismatch (a field present with the wrong type) — and decline anything else. Either way the
        // node matched an impl, so it is a complete non-terminal and is cached.
        let result = match has_field_projection_mismatch(tcx, erased) {
            Some((field_ref, expected)) => {
                projection_result(tcx, erased, leaf_ref, context, field_ref, expected, self_fp)
            }
            None => SubResult::empty(self_fp),
        };
        return cache_if_complete(cache, key, result);
    } else {
        unmet
    };

    // Non-terminal: merge the children's subtrees, prepending this node's label to each sub-chain.
    let node_label = label_for(tcx, erased, context);
    let child_prefix: Vec<_> = prefix
        .iter()
        .copied()
        .chain(std::iter::once(erased))
        .collect();
    let mut causes: Vec<SubCause> = Vec::new();
    let mut reachable: FxHashSet<Fingerprint> = FxHashSet::default();
    reachable.insert(self_fp);
    let mut incomplete = false;
    for child in children {
        let sub = resolve_node(
            tcx,
            cache,
            child,
            Some(leaf_ref),
            context,
            &child_prefix,
            depth + 1,
        );
        incomplete |= sub.incomplete;
        reachable.extend(sub.reachable);
        for mut sc in sub.causes {
            if let Some(label) = &node_label {
                // Prepend this node's label so the stored sub-chain is rooted at this node.
                sc.labels.insert(0, label.clone());
            }
            causes.push(sc);
        }
    }
    cache_if_complete(
        cache,
        key,
        SubResult {
            causes,
            reachable,
            incomplete,
        },
    )
}

/// Store `result` under `key` when it is complete (untainted by a cycle or depth cut), and return
/// it. An incomplete subtree is never cached, so a later reuse can never under-report a branch a
/// guard curtailed.
fn cache_if_complete(cache: &ResolveCache, key: NodeKey, result: SubResult) -> SubResult {
    if !result.incomplete {
        cache.insert(key, result.clone());
    }
    result
}

/// Build the sub-result for a terminal leaf `leaf_ref` (a `HasField`, an impl-less bound, or a
/// foreign bound with nothing to follow). Drops a leaf still carrying a call-site placeholder (an
/// unknowable `_: Send`) and a non-reportable plumbing dead-end, in both cases as a complete
/// no-cause result. Not cached — the classification reads `parent`, which lies outside the leaf.
fn terminal_result<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitPredicate<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    context: Ty<'tcx>,
    self_fp: Fingerprint,
) -> SubResult {
    // A leaf still carrying one of the seed's unknown-parameter placeholders (the call-site anchor's
    // stand-in for a call's inferred input) is a bound on a type the recovery could not know, so it
    // is dropped rather than reported as a fabricated `_: Send`.
    if erased.has_placeholders() {
        return SubResult::empty(self_fp);
    }
    // A path that bottoms out on pure wiring plumbing (a routing dead-end) is not a root cause.
    if !is_reportable_leaf(tcx, leaf_ref, context, parent) {
        return SubResult::empty(self_fp);
    }
    let leaf = classify_leaf(tcx, leaf_ref, context, parent, None);
    let label = dependency_tree_leaf(&leaf);
    let mut reachable = FxHashSet::default();
    reachable.insert(self_fp);
    SubResult {
        causes: vec![SubCause {
            leaf,
            labels: vec![label],
        }],
        reachable,
        incomplete: false,
    }
}

/// Build the sub-result for a field-type mismatch: the impl matched with every trait-clause holding,
/// but a `HasField::Value` projection is wrong. The node itself becomes a chain hop (its label) and
/// the field's `HasField` ref is the terminal leaf, carrying the expected type. `parent_ref` (the
/// node's own trait) is the field leaf's parent.
fn projection_result<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitPredicate<'tcx>,
    parent_ref: ty::TraitRef<'tcx>,
    context: Ty<'tcx>,
    field_ref: ty::TraitRef<'tcx>,
    expected: Ty<'tcx>,
    self_fp: Fingerprint,
) -> SubResult {
    let leaf_poly: ty::PolyTraitPredicate<'tcx> = ty::Binder::dummy(field_ref).upcast(tcx);
    if leaf_poly.has_placeholders() {
        return SubResult::empty(self_fp);
    }
    let parent = Some(parent_ref);
    if !is_reportable_leaf(tcx, field_ref, context, parent) {
        return SubResult::empty(self_fp);
    }
    let leaf = classify_leaf(tcx, field_ref, context, parent, Some(expected));
    let leaf_label = dependency_tree_leaf(&leaf);
    let mut labels = Vec::new();
    if let Some(node_label) = label_for(tcx, erased, context) {
        labels.push(node_label);
    }
    labels.push(leaf_label);
    let mut reachable = FxHashSet::default();
    reachable.insert(self_fp);
    reachable.insert(pred_fingerprint(tcx, leaf_poly));
    SubResult {
        causes: vec![SubCause { leaf, labels }],
        reachable,
        incomplete: false,
    }
}
