//! Resolving one node of the dependency graph, memoized at every node.

use cargo_cgp_error_processing::{ChainNode, WiringOverflow};
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::{self, Ty, TypeVisitableExt};

use crate::resolve::cache::{NodeKey, ResolveCache, SubCause, SubResult, pred_fingerprint};
use crate::resolve::classify::wiring_without_provider_for;
use crate::resolve::ctx::ResolveCtx;
use crate::resolve::label::label_for;
use crate::resolve::query::{
    holds, impl_where_obligations, is_consumer_trait, is_context_adt, misdirected_impl_clauses,
    projection_mismatch, unmet_projection,
};
use crate::resolve::walk::{
    cause_result, delegated_table_lookups, growing_lookup, is_context_side, is_descendable,
    is_has_field, is_other_context_provider, is_workaround_plumbing, missing_context_wiring,
    projection_result, provider_step, terminal_result,
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
    // Depth cap: a divergent wiring the cycle guard cannot catch (obligations that keep growing
    // without repeating) is cut here. The cut flags the subtree incomplete so it is never cached.
    if depth > MAX_DEPTH {
        return SubResult::cut(growing_lookup(tcx, prefix));
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
        return SubResult::cut(WiringOverflow::Cycle {
            closing: parent.and_then(|parent| {
                let grandparent = prefix.len().checked_sub(2).map(|at| prefix[at]);
                provider_step(tcx, grandparent, parent, erased)
            }),
        });
    }

    // Cross-context handoff: a consumer obligation whose `Self` is a *different* local context is the
    // cross-context dependency shape — one context's wiring depends on a concrete other context. Re-root
    // the subtree at that context, so its nodes are labeled and classified against the context they are
    // actually about: a consumer node reads `for context Inner` (not a plain foreign bound), its
    // delegation-routing hop is recognized as routing and dropped, and its getter leaf decodes to a
    // missing field rather than an opaque bound. Re-rooting also keys this node identically to that
    // context's own walk, so the two share the cache. This only fires cross-context; in single-context
    // wiring every consumer obligation is already on the root context.
    let context = if erased.skip_binder().self_ty() != context
        && is_context_adt(tcx, erased.skip_binder().self_ty())
        && is_consumer_trait(tcx, erased.skip_binder().trait_ref.def_id)
    {
        erased.skip_binder().self_ty()
    } else {
        context
    };

    // Interior-cache consult. Only complete non-terminal nodes are ever stored, so this hits only a
    // fully-explored subtree; reuse it only when no current ancestor lies inside it, since otherwise
    // splicing it would keep a branch a fresh walk's cycle guard would cut here.
    let key = NodeKey::new(tcx, erased, context);
    if let Some(cached) = cx.cache.get(&key) {
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

    // An unmet `DelegateComponent` on the **context** is the missing-wiring leaf, whatever impl
    // nominally matches it. Normally none does and the terminal branch below catches it, but a
    // context that joins a namespace carries a blanket `DelegateComponent<__Key__>` forwarding that
    // unifies with *every* key, so the descent would follow it into the namespace's own lookup
    // machinery (`ConcatPath`, `Sized` on the unresolved key) and bottom out on plumbing instead.
    // Descending can never find anything better: had the context wired this key the obligation would
    // hold and be pruned before becoming a node at all, so reaching here means the entry is simply
    // absent — which is the cause to report. The one exception is an entry that *delegates* the key
    // to another table (`Key -> Table`): the context does carry it, and the failure is that table's
    // lookup, so the walk descends there instead of blaming the context.
    let mut delegated = Vec::new();
    if let Some(missing) = missing_context_wiring(tcx, erased, context) {
        delegated = delegated_table_lookups(tcx, erased, context);
        if delegated.is_empty() {
            return terminal_result(
                tcx,
                missing,
                missing.skip_binder().trait_ref,
                parent,
                context,
                self_fp,
            );
        }
    }

    let descendable = is_descendable(tcx, erased, context);

    // A provider whose `#[cgp_impl]` header names the consumer trait has no provider-trait impl, so
    // the obligation would end at "not a provider", a consequence of the header `[CGP-E032]` already
    // reports. That impl still states what the provider needs from the context, and a need that
    // fails is a separate mistake that outlives the header fix, so the walk descends into it.
    let mut misdirected: Vec<ty::PolyTraitClause<'tcx>> = Vec::new();
    if delegated.is_empty()
        && let Some(clauses) = misdirected_impl_clauses(tcx, leaf_ref)
    {
        misdirected = clauses
            .iter()
            .filter_map(|clause| clause.as_trait_clause())
            .filter(|nested| !holds(tcx, *nested) && !is_workaround_plumbing(tcx, *nested))
            .collect();
        if misdirected.is_empty()
            && let Some(mismatch) = unmet_projection(tcx, clauses)
        {
            let result = projection_result(tcx, erased, leaf_ref, context, mismatch, self_fp);
            return cache_if_complete(cx.cache, key, result);
        }
    }

    let Some(children) = (if !misdirected.is_empty() {
        Some(misdirected)
    } else if delegated.is_empty() {
        impl_where_obligations(tcx, erased)
    } else {
        Some(delegated)
    }) else {
        // No impl satisfies `pred` at all — `pred` is itself the terminal root-cause bound.
        return terminal_result(tcx, erased, leaf_ref, parent, context, self_fp);
    };

    let failing: Vec<_> = children
        .into_iter()
        .filter(|nested| !holds(tcx, *nested))
        .collect();
    // Drop the check-trait scaffolding wherever it appears as a dependency: the generated blanket
    // impls carry a `CanUseComponent`/`IsProviderFor` bound *beside* the real consumer or
    // provider-trait obligation, and only the latter is walked — the `IsProviderFor` bound just
    // re-states the provider's `where` clause, which the real provider impl already carries.
    let unmet: Vec<_> = failing
        .iter()
        .copied()
        .filter(|nested| !is_workaround_plumbing(tcx, *nested))
        .collect();

    // Decide the children to descend, or return a terminal / projection result directly.
    let children: Vec<ty::PolyTraitClause<'tcx>> = if !descendable {
        // A foreign-type bound is normally the terminal root cause, and the descent must not wander
        // into whatever `std` blanket impl happens to satisfy it. Two exceptions are followed: a CGP
        // getter or blanket trait on a non-context type whose blanket impl depends on the *context*
        // (so the real cause surfaces and de-duplicates), and a same-trait recursion over a
        // type-level list (a record's `Cons<..>: HandleMapEntry<..>` whose tail is another
        // `Cons<.., Nil>: …`). A foreign `f64: FnPtr` step is neither, so the bound stays the leaf,
        // and so is a blanket impl that only reaches another context's provider: a dispatch payload
        // missing its own impl of the dispatched trait is the cause, not the enum-matching blanket
        // its type cannot use.
        let this_trait = erased.def_id();
        let followable: Vec<_> = unmet
            .into_iter()
            .filter(|nested| {
                nested.def_id() == this_trait
                    || (is_context_side(tcx, *nested, context)
                        && !is_other_context_provider(tcx, *nested, context))
            })
            .collect();
        if followable.is_empty() {
            return terminal_result(tcx, erased, leaf_ref, parent, context, self_fp);
        }
        followable
    } else if unmet.is_empty() {
        // Only scaffolding fails: the context's own `IsProviderFor` beside its delegate, which a
        // hand-written `DelegateComponent` impl leaves out.
        if let Some(leaf) = wiring_without_provider_for(tcx, &failing, context) {
            let result = cause_result(tcx, erased, context, leaf, self_fp);
            return cache_if_complete(cx.cache, key, result);
        }
        // Matched an impl, yet every trait-clause `where`-obligation holds: the fault is a projection
        // the trait-clause walk cannot see — a `HasField::Value` mismatch (a field present with the
        // wrong type), or any other associated type the owner supplies differently from what the impl
        // requires (a CGP abstract type, typically). Either way the node matched an impl, so it is a
        // complete non-terminal and is cached.
        let result = match projection_mismatch(tcx, erased) {
            Some(mismatch) => projection_result(tcx, erased, leaf_ref, context, mismatch, self_fp),
            None => SubResult::empty(self_fp),
        };
        return cache_if_complete(cx.cache, key, result);
    } else {
        unmet
    };

    // Non-terminal: merge the children's subtrees, prepending this node's hop to each sub-path.
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
    cache_if_complete(
        cx.cache,
        key,
        SubResult {
            causes,
            reachable,
            incomplete,
            cut,
        },
    )
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
