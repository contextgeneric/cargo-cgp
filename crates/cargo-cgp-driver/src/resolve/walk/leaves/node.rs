//! Resolving one node of the dependency graph, memoized at every node.

use cargo_cgp_error_processing::{ChainNode, WiringOverflow};
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt};

use crate::resolve::cache::{NodeKey, ResolveCache, SubCause, SubResult, pred_fingerprint};
use crate::resolve::ctx::ResolveCtx;
use crate::resolve::label::label_for;
use crate::resolve::query::{is_consumer_trait, is_context_adt};
use crate::resolve::walk::{
    NodeStep, growing_lookup, is_has_field, provider_step, select_children, terminal_result,
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
pub const MAX_DEPTH: u32 = 256;

/// Resolve one node of the dependency graph to its owned, node-rooted sub-causes, memoized in
/// the context's cache. `pred` is the (already region-erased) failing obligation, `parent` the trait of the
/// obligation directly above it (needed only to classify a terminal leaf), `context` the root
/// context, and `prefix` the chain of ancestors above `pred` (for the cycle guard and the reuse
/// disjointness check). A `HasField`, or an obligation with no satisfying impl, or a foreign bound
/// with nothing context-side to follow, is a terminal leaf; anything else descends into the unmet
/// `where`-obligations of the impl that would satisfy it. Only complete (untainted) non-terminal
/// nodes are cached.
pub fn resolve_node<'tcx>(
    cx: ResolveCtx<'_, 'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    context: Ty<'tcx>,
    prefix: &[ty::PolyTraitClause<'tcx>],
    depth: u32,
) -> SubResult {
    let tcx = cx.tcx;
    let erased = tcx.erase_and_anonymize_regions(pred);
    if let Some(cut) = guard_cut(tcx, erased, parent, prefix, depth) {
        return cut;
    }
    let context = rooted_context(tcx, erased, context);
    let key = NodeKey::new(tcx, erased, context);
    if let Some(cached) = reusable_cached(cx, &key, prefix) {
        return cached;
    }

    let self_fp = pred_fingerprint(tcx, erased);
    let leaf_ref = erased.skip_binder().trait_ref;

    // An obligation holding a `{type error}` names a type an earlier error left unresolved. Such a
    // type unifies with anything, so walking into it can only invent causes or loops; the earlier
    // error is the cause, and the branch ends here with nothing to report.
    if erased.references_error() {
        return SubResult::empty(self_fp);
    }

    // A `HasField` completes a path directly — a terminal leaf, not cached (its classification can
    // read its parent, which lies outside a leaf-rooted subtree, and it is cheap to re-derive).
    if is_has_field(tcx, erased) {
        return terminal_result(tcx, erased, leaf_ref, parent, context, self_fp);
    }

    match select_children(tcx, erased, parent, context, self_fp) {
        NodeStep::Terminal(result) => result,
        NodeStep::Complete(result) => cache_if_complete(cx.cache, key, result),
        NodeStep::Descend(children) => {
            let result = merge_children(cx, erased, context, prefix, depth, children, self_fp);
            cache_if_complete(cx.cache, key, result)
        }
    }
}

/// The cut result of a node the walk must not enter: past the depth cap, or an obligation already
/// among its ancestors.
pub fn guard_cut<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    prefix: &[ty::PolyTraitClause<'tcx>],
    depth: u32,
) -> Option<SubResult> {
    // Depth cap: a divergent wiring the cycle guard cannot catch (obligations that keep growing
    // without repeating) is cut here. The cut flags the subtree incomplete so it is never cached.
    if depth > MAX_DEPTH {
        return Some(SubResult::cut(growing_lookup(tcx, prefix)));
    }

    // Cycle guard: if this exact obligation already appears among its own ancestors, the wiring
    // loops (a `UseContext` cycle routes `Ctx: CanUseComponent<C>` straight back to itself), so this
    // branch carries no new root cause — stop rather than descend the loop. Regions are erased so a
    // loop that only differs by lifetime is still seen. The cut flags the subtree incomplete, which
    // propagates up so no ancestor whose result depended on the cut is cached.
    if prefix
        .iter()
        .any(|ancestor| tcx.erase_and_anonymize_regions(*ancestor) == erased)
    {
        return Some(SubResult::cut(WiringOverflow::Cycle {
            closing: parent.and_then(|parent| {
                let grandparent = prefix.len().checked_sub(2).map(|at| prefix[at]);
                provider_step(tcx, grandparent, parent, erased)
            }),
        }));
    }
    None
}

/// The context a node is about, given the `context` above it.
///
/// Cross-context handoff: a consumer obligation whose `Self` is a *different* local context is the
/// cross-context dependency shape — one context's wiring depends on a concrete other context. Re-root
/// the subtree at that context, so its nodes are labeled and classified against the context they are
/// actually about: a consumer node reads `for context Inner` (not a plain foreign bound), its
/// delegation-routing hop is recognized as routing and dropped, and its getter leaf decodes to a
/// missing field rather than an opaque bound. Re-rooting also keys this node identically to that
/// context's own walk, so the two share the cache. This only fires cross-context; in single-context
/// wiring every consumer obligation is already on the root context.
pub fn rooted_context<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> Ty<'tcx> {
    if erased.skip_binder().self_ty() != context
        && is_context_adt(tcx, erased.skip_binder().self_ty())
        && is_consumer_trait(tcx, erased.skip_binder().trait_ref.def_id)
    {
        erased.skip_binder().self_ty()
    } else {
        context
    }
}

/// The cached result for `key`, when it may be reused under these ancestors.
///
/// Interior-cache consult. Only complete non-terminal nodes are ever stored, so this hits only a
/// fully-explored subtree; reuse it only when no current ancestor lies inside it, since otherwise
/// splicing it would keep a branch a fresh walk's cycle guard would cut here.
pub fn reusable_cached<'tcx>(
    cx: ResolveCtx<'_, 'tcx>,
    key: &NodeKey,
    prefix: &[ty::PolyTraitClause<'tcx>],
) -> Option<SubResult> {
    let tcx = cx.tcx;
    let cached = cx.cache.get(key)?;
    let reusable = prefix.iter().all(|ancestor| {
        !cached.reachable.contains(&pred_fingerprint(
            tcx,
            tcx.erase_and_anonymize_regions(*ancestor),
        ))
    });
    reusable.then_some(cached)
}

/// Merge the children's subtrees, prepending this node's hop to each sub-path.
pub fn merge_children<'tcx>(
    cx: ResolveCtx<'_, 'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
    prefix: &[ty::PolyTraitClause<'tcx>],
    depth: u32,
    children: Vec<ty::PolyTraitClause<'tcx>>,
    self_fp: Fingerprint,
) -> SubResult {
    let tcx = cx.tcx;
    let leaf_ref = erased.skip_binder().trait_ref;
    let node_hop = label_for(tcx, erased, context);
    let child_prefix: Vec<_> = prefix
        .iter()
        .copied()
        .chain(std::iter::once(erased))
        .collect();
    let mut causes: Vec<SubCause> = Vec::new();
    let mut reachable: FxHashSet<Fingerprint> = FxHashSet::default();
    reachable.insert(self_fp);
    let mut incomplete = false;
    let mut cut: Option<WiringOverflow> = None;
    for child in children {
        let sub = resolve_node(cx, child, Some(leaf_ref), context, &child_prefix, depth + 1);
        incomplete |= sub.incomplete;
        // A cycle explains a lookup that never resolves better than a depth cut does, so it wins.
        match (&cut, sub.cut) {
            (None, found @ Some(_))
            | (Some(WiringOverflow::Growing { .. }), found @ Some(WiringOverflow::Cycle { .. })) => {
                cut = found
            }
            _ => {}
        }
        reachable.extend(sub.reachable);
        for mut sc in sub.causes {
            if let Some(hop) = &node_hop {
                // Prepend this node's hop so the stored sub-path is rooted at this node.
                sc.path.insert(0, ChainNode::Hop(hop.clone()));
            }
            causes.push(sc);
        }
    }
    SubResult {
        causes,
        reachable,
        incomplete,
        cut,
    }
}

/// Store `result` under `key` when it is complete (untainted by a cycle or depth cut), and return
/// it. An incomplete subtree is never cached, so a later reuse can never under-report a branch a
/// guard curtailed.
pub fn cache_if_complete(cache: &ResolveCache, key: NodeKey, result: SubResult) -> SubResult {
    if !result.incomplete {
        cache.insert(key, result.clone());
    }
    result
}
