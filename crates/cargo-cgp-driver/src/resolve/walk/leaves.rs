//! Descending the failing obligation to every terminal root-cause leaf.

use cargo_cgp_error_processing::{Causes, ChainNode, Leaf, OverflowStep, Resolved, WiringOverflow};
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt, Upcast as _};

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::cache::{NodeKey, ResolveCache, SubCause, SubResult, pred_fingerprint};
use crate::resolve::cgp_item::{
    is_cgp_item, is_consumer_trait, is_local_adt, is_provider_trait, trim_unknown_path_tail,
};
use crate::resolve::classify::{classify_leaf, is_reportable_leaf, wiring_without_provider_for};
use crate::resolve::label::{label_for, redirect_lookup_args, render_ty, trait_generics};
use crate::resolve::walk::{
    ProjectionMismatch, holds, impl_where_obligations, is_context_side, is_descendable,
    is_has_field, is_other_context_provider, is_workaround_plumbing, may_hold_for_some_unknown,
    misdirected_impl_clauses, projection_mismatch, unmet_projection,
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
/// failure at many sites (all seeding the same obligation), and a shared trait is a diamond
/// reached from several parents, so each distinct obligation is resolved once and reused. See
/// `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.
pub(crate) fn resolve_leaves<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    top: ty::PolyTraitClause<'tcx>,
) -> Option<Resolved> {
    // Erase the seed's free regions up front, exactly as every descendant obligation is erased by
    // [`impl_where_obligations`]. A lifetime-parameterized check entry (`(Life<'a>, str)`) seeds an
    // obligation carrying the check impl's own `'a`, and leaving it unerased would make the seed the
    // one chain entry whose `self_ty == context` comparisons fail, mislabeling the consumer node.
    let top = tcx.erase_and_anonymize_regions(top);
    let context = top.skip_binder().self_ty();
    compute_leaves(tcx, cache, top, context)
}

/// Walk the dependency graph of `top`, the provider obligation `Provider: ProviderTrait<Ctx, …>` a
/// `#[check_providers]` entry or a call to the provider's own method asserts, with `context` as the
/// walk's root context. The result is the provider class: its trait is the provider trait (named
/// without the provider and the context), and its [`provider`](Resolved::provider) is that provider. `None` when no branch
/// reaches a resolvable leaf.
pub(crate) fn resolve_provider_leaves<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    top: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> Option<Resolved> {
    let top = tcx.erase_and_anonymize_regions(top);
    let context = tcx.erase_and_anonymize_regions(context);
    let mut resolved = compute_leaves(tcx, cache, top, context)?;
    let top_ref = top.skip_binder().trait_ref;
    resolved.consumers = vec![format!(
        "{}{}",
        tcx.item_name(top_ref.def_id),
        trait_generics(tcx, top_ref, 2)
    )];
    resolved.provider = Some(render_ty(tcx, top_ref.self_ty()));
    Some(resolved)
}

/// Why walking `top` was cut short, when a guard cut it: the obligation met again (a cycle in the
/// wiring) or the depth cap reached without a repeat (a lookup that grows). `None` when the walk ran
/// to its leaves, as it does through a type the compiler could not resolve.
pub(crate) fn walk_overflow<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    top: ty::PolyTraitClause<'tcx>,
) -> Option<WiringOverflow> {
    let top = tcx.erase_and_anonymize_regions(top);
    let context = top.skip_binder().self_ty();
    resolve_node(tcx, cache, top, None, context, &[], 0).cut
}

/// The provider step `parent` that requires `required`, when `parent` is the provider-trait
/// obligation of a provider a wiring entry chose (not the context's delegation routing, whose
/// `Self` is the context). `grandparent` is the obligation above `parent`: the entry's routing, on
/// the context for a direct entry or on a `RedirectLookup` for an `open` or namespace entry. A
/// provider a higher-order provider holds inside it, such as a dispatcher's inner `UseContext`, is
/// below another provider instead, and no entry names it.
fn provider_step<'tcx>(
    tcx: TyCtxt<'tcx>,
    grandparent: Option<ty::PolyTraitClause<'tcx>>,
    parent: ty::TraitRef<'tcx>,
    required: ty::PolyTraitClause<'tcx>,
) -> Option<OverflowStep> {
    let required = required.skip_binder().trait_ref;
    let context = required.self_ty();
    if !is_provider_trait(tcx, parent.def_id) || parent.self_ty() == context {
        return None;
    }
    let routing = grandparent?.skip_binder().trait_ref;
    let chosen_by_entry = routing.def_id == parent.def_id
        && (routing.self_ty() == context || redirect_lookup_args(tcx, routing.self_ty()).is_some());
    if !chosen_by_entry {
        return None;
    }
    Some(OverflowStep {
        provider: tcx
            .erase_and_anonymize_regions(parent.self_ty())
            .to_string(),
        provider_trait: format!(
            "{}{}",
            tcx.item_name(parent.def_id),
            trait_generics(tcx, parent, 2)
        ),
        context: context.to_string(),
        requires: format!(
            "{}{}",
            tcx.item_name(required.def_id),
            trait_generics(tcx, required, 1)
        ),
    })
}

/// The growing lookup a depth cut ends: the first obligation in `prefix` that a later one repeats
/// with other arguments on the same type, the later one being the first larger requirement, and the
/// provider step just above it.
fn growing_lookup<'tcx>(tcx: TyCtxt<'tcx>, prefix: &[ty::PolyTraitClause<'tcx>]) -> WiringOverflow {
    let same_kind = |a: &ty::PolyTraitClause<'tcx>, b: &ty::PolyTraitClause<'tcx>| {
        a.def_id() == b.def_id() && a.skip_binder().self_ty() == b.skip_binder().self_ty()
    };
    let larger = prefix.iter().enumerate().find_map(|(at, first)| {
        prefix[at + 1..]
            .iter()
            .position(|later| same_kind(first, later) && later != first)
            .map(|offset| at + 1 + offset)
    });
    let Some(larger) = larger else {
        return WiringOverflow::Growing {
            larger: None,
            growing: None,
        };
    };
    let larger_ref = prefix[larger].skip_binder().trait_ref;
    WiringOverflow::Growing {
        larger: Some(format!(
            "{}{}",
            tcx.item_name(larger_ref.def_id),
            trait_generics(tcx, larger_ref, 1)
        )),
        growing: prefix[..larger].split_last().and_then(|(parent, above)| {
            provider_step(
                tcx,
                above.last().copied(),
                parent.skip_binder().trait_ref,
                prefix[larger],
            )
        }),
    }
}

/// Fold the root node's owned sub-causes into a [`Resolved`]: group by leaf into one [`Cause`] per
/// distinct root cause, each holding every path that reaches it (so a shared trait's diamond
/// survives to the renderer). Merging happens in the rustc-free
/// [dependency graph](cargo_cgp_error_processing::DependencyGraph) at render time, not here. `top`
/// is already region-erased and `context` is its self type.
fn compute_leaves<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    top: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> Option<Resolved> {
    let sub = resolve_node(tcx, cache, top, None, context, &[], 0);

    // One cause per distinct leaf; a leaf reached by several paths keeps each path, so the graph
    // can render the convergence rather than dropping every path but the first.
    let causes = Causes::from_sub_chains(sub.causes.into_iter().map(|sc| (sc.leaf, sc.path)));

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
        data_operation: false,
        provider: None,
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
    pred: ty::PolyTraitClause<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    context: Ty<'tcx>,
    prefix: &[ty::PolyTraitClause<'tcx>],
    depth: u32,
) -> SubResult {
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
        && is_local_adt(erased.skip_binder().self_ty())
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
            return cache_if_complete(cache, key, result);
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
            return cache_if_complete(cache, key, result);
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
        return cache_if_complete(cache, key, result);
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
        cache,
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
fn cache_if_complete(cache: &ResolveCache, key: NodeKey, result: SubResult) -> SubResult {
    if !result.incomplete {
        cache.insert(key, result.clone());
    }
    result
}

/// The unmet `DelegateComponent` lookups on *another* table that the context's own entry for
/// `pred`'s key forwards to, the `where Table: DelegateComponent<Key>` bound a `Key -> Table` entry
/// generates. Empty when the context has no such entry, so its missing wiring stands.
fn delegated_table_lookups<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> Vec<ty::PolyTraitClause<'tcx>> {
    let Some(children) = impl_where_obligations(tcx, pred) else {
        return Vec::new();
    };
    children
        .into_iter()
        .filter(|child| {
            let trait_ref = child.skip_binder().trait_ref;
            is_cgp_item(
                tcx,
                trait_ref.def_id,
                DELEGATE_COMPONENT_TRAIT,
                CGP_COMPONENT_CRATE,
            ) && tcx.erase_and_anonymize_regions(trait_ref.self_ty()) != context
                && !holds(tcx, *child)
        })
        .collect()
}

/// The unmet `DelegateComponent` obligation to report when `pred` is one **on the context** — with
/// any trailing unknown segments trimmed off a redirect-path key — or `None` when `pred` is not one.
///
/// The trim is what makes a call-site-anchored redirect reportable. A `RedirectLookup` keys on the
/// path *plus the component's parameters*, so a lookup recovered from a call whose input is inferred
/// carries that placeholder as the path's last segment: the leaf would name an entry the programmer
/// could not write, and would be dropped as unknowable before it was ever shown. Trimming leaves the
/// path they can actually wire ([`trim_unknown_path_tail`]). The trimmed leaf claims the context has
/// no entry under that path for *any* value of the unknown, so it is kept only when the lookup may
/// hold for none ([`may_hold_for_some_unknown`]); otherwise `pred` is returned untrimmed, and the
/// placeholder it carries drops it.
fn missing_context_wiring<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> Option<ty::PolyTraitClause<'tcx>> {
    let trait_ref = pred.skip_binder().trait_ref;
    if !is_cgp_item(
        tcx,
        trait_ref.def_id,
        DELEGATE_COMPONENT_TRAIT,
        CGP_COMPONENT_CRATE,
    ) {
        return None;
    }
    if tcx.erase_and_anonymize_regions(trait_ref.self_ty()) != context {
        return None;
    }

    let key = trait_ref.args.type_at(1);
    let Some(trimmed) = trim_unknown_path_tail(tcx, key) else {
        return Some(pred);
    };
    if may_hold_for_some_unknown(tcx, pred) {
        return Some(pred);
    }
    let args: [ty::GenericArg<'tcx>; 2] = [trait_ref.self_ty().into(), trimmed.into()];
    let trait_ref = ty::TraitRef::new(tcx, trait_ref.def_id, args);
    Some(ty::Binder::dummy(trait_ref).upcast(tcx))
}

/// Build the sub-result for a terminal leaf `leaf_ref` (a `HasField`, an impl-less bound, or a
/// foreign bound with nothing to follow). Drops a leaf still carrying a call-site placeholder (an
/// unknowable `_: Send`) and a non-reportable plumbing dead-end, in both cases as a complete
/// no-cause result. Not cached — the classification reads `parent`, which lies outside the leaf.
fn terminal_result<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
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
    let mut path = Vec::new();
    // A seed that is itself the cause (a data operation with no impl at all, such as a finalize on
    // a builder with a field unset) heads the chain with its own step, so the chain names the
    // operation the cause is about rather than restating the cause alone.
    if parent.is_none()
        && let Some(hop) = label_for(tcx, erased, context)
    {
        path.push(ChainNode::Hop(hop));
    }
    path.push(ChainNode::Leaf(leaf.clone()));
    let mut reachable = FxHashSet::default();
    reachable.insert(self_fp);
    SubResult {
        causes: vec![SubCause { path, leaf }],
        reachable,
        incomplete: false,
        cut: None,
    }
}

/// Build the sub-result for a cause the node itself states, already classified as `leaf`: the node
/// is a chain hop (its label, when it has one) and `leaf` ends the path.
fn cause_result<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
    leaf: Leaf,
    self_fp: Fingerprint,
) -> SubResult {
    let mut path = Vec::new();
    if let Some(node_hop) = label_for(tcx, erased, context) {
        path.push(ChainNode::Hop(node_hop));
    }
    path.push(ChainNode::Leaf(leaf.clone()));
    let mut reachable = FxHashSet::default();
    reachable.insert(self_fp);
    SubResult {
        causes: vec![SubCause { leaf, path }],
        reachable,
        incomplete: false,
        cut: None,
    }
}

/// Build the sub-result for a projection mismatch: the impl matched with every trait-clause holding,
/// but an associated type it projects through carries the wrong type. The node itself becomes a chain
/// hop (its label) and the projection's own trait ref is the terminal leaf, carrying the mismatch.
/// `parent_ref` (the node's own trait) is that leaf's parent.
fn projection_result<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    parent_ref: ty::TraitRef<'tcx>,
    context: Ty<'tcx>,
    mismatch: ProjectionMismatch<'tcx>,
    self_fp: Fingerprint,
) -> SubResult {
    let leaf_ref = mismatch.trait_ref;
    let leaf_poly: ty::PolyTraitClause<'tcx> = ty::Binder::dummy(leaf_ref).upcast(tcx);
    if leaf_poly.has_placeholders() || leaf_poly.references_error() {
        return SubResult::empty(self_fp);
    }
    let parent = Some(parent_ref);
    if !is_reportable_leaf(tcx, leaf_ref, context, parent) {
        return SubResult::empty(self_fp);
    }
    let leaf = classify_leaf(tcx, leaf_ref, context, parent, Some(mismatch));
    let mut path = Vec::new();
    if let Some(node_hop) = label_for(tcx, erased, context) {
        path.push(ChainNode::Hop(node_hop));
    }
    path.push(ChainNode::Leaf(leaf.clone()));
    let mut reachable = FxHashSet::default();
    reachable.insert(self_fp);
    reachable.insert(pred_fingerprint(tcx, leaf_poly));
    SubResult {
        causes: vec![SubCause { leaf, path }],
        reachable,
        incomplete: false,
        cut: None,
    }
}
