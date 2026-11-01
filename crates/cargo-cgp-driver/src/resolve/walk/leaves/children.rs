//! Choosing what a node of the dependency graph descends into, or the result it ends in.

use rustc_data_structures::fingerprint::Fingerprint;
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::resolve::cache::SubResult;
use crate::resolve::classify::wiring_without_provider_for;
use crate::resolve::query::{
    holds, impl_where_obligations, misdirected_impl_clauses, projection_mismatch, unmet_projection,
};
use crate::resolve::walk::{
    cause_result, delegated_table_lookups, is_context_side, is_descendable,
    is_other_context_provider, is_workaround_plumbing, missing_context_wiring, projection_result,
    terminal_result,
};

/// What a node does after its guards.
pub enum NodeStep<'tcx> {
    /// It ends in a terminal leaf, which is not cached: its classification can read its parent,
    /// which lies outside a leaf-rooted subtree.
    Terminal(SubResult),
    /// It matched an impl and ends here, a complete non-terminal the cache may store.
    Complete(SubResult),
    /// It descends into these unmet obligations.
    Descend(Vec<ty::PolyTraitClause<'tcx>>),
}

/// Choose what the node `erased` does: the context-wiring leaf, a misdirected provider's own needs, or
/// the unmet `where`-obligations of the impl that would satisfy it.
///
/// An unmet `DelegateComponent` on the **context** is the missing-wiring leaf, whatever impl
/// nominally matches it. Normally none does and the no-impl terminal catches it, but a
/// context that joins a namespace carries a blanket `DelegateComponent<__Key__>` forwarding that
/// unifies with *every* key, so the descent would follow it into the namespace's own lookup
/// machinery (`ConcatPath`, `Sized` on the unresolved key) and bottom out on plumbing instead.
/// Descending can never find anything better: had the context wired this key the obligation would
/// hold and be pruned before becoming a node at all, so reaching here means the entry is simply
/// absent, which is the cause to report. The one exception is an entry that *delegates* the key
/// to another table (`Key -> Table`): the context does carry it, and the failure is that table's
/// lookup, so the walk descends there instead of blaming the context.
///
/// A provider whose `#[cgp_impl]` header names the consumer trait has no provider-trait impl, so
/// the obligation would end at "not a provider", a consequence of the header `[CGP-E032]` already
/// reports. That impl still states what the provider needs from the context, and a need that
/// fails is a separate mistake that outlives the header fix, so the walk descends into it.
pub fn select_children<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    context: Ty<'tcx>,
    self_fp: Fingerprint,
) -> NodeStep<'tcx> {
    let leaf_ref = erased.skip_binder().trait_ref;

    let mut delegated = Vec::new();
    if let Some(missing) = missing_context_wiring(tcx, erased, context) {
        delegated = delegated_table_lookups(tcx, erased, context);
        if delegated.is_empty() {
            return NodeStep::Terminal(terminal_result(
                tcx,
                missing,
                missing.skip_binder().trait_ref,
                parent,
                context,
                self_fp,
            ));
        }
    }

    let descendable = is_descendable(tcx, erased, context);

    let mut misdirected: Vec<ty::PolyTraitClause<'tcx>> = Vec::new();
    if delegated.is_empty()
        && let Some(clauses) = misdirected_impl_clauses(tcx, leaf_ref)
    {
        let failing: Vec<_> = clauses
            .iter()
            .filter_map(|clause| clause.as_trait_clause())
            .filter(|nested| !holds(tcx, *nested))
            .collect();
        misdirected = unmet_clauses(tcx, &failing);
        if misdirected.is_empty()
            && let Some(mismatch) = unmet_projection(tcx, clauses)
        {
            return NodeStep::Complete(projection_result(
                tcx, erased, leaf_ref, context, mismatch, self_fp,
            ));
        }
    }

    let Some(children) = (if !misdirected.is_empty() {
        Some(misdirected)
    } else if delegated.is_empty() {
        impl_where_obligations(tcx, erased)
    } else {
        Some(delegated)
    }) else {
        // No impl satisfies `pred` at all: `pred` is itself the terminal root-cause bound.
        return NodeStep::Terminal(terminal_result(
            tcx, erased, leaf_ref, parent, context, self_fp,
        ));
    };

    let failing: Vec<_> = children
        .into_iter()
        .filter(|nested| !holds(tcx, *nested))
        .collect();
    let unmet = unmet_clauses(tcx, &failing);

    if !descendable {
        followable_children(tcx, erased, parent, context, self_fp, unmet)
    } else if unmet.is_empty() {
        NodeStep::Complete(result_without_unmet_clause(
            tcx, erased, context, &failing, self_fp,
        ))
    } else {
        NodeStep::Descend(unmet)
    }
}

/// `failing` without the check-trait scaffolding, wherever it appears as a dependency: the generated
/// blanket impls carry a `CanUseComponent`/`IsProviderFor` bound *beside* the real consumer or
/// provider-trait obligation, and only the latter is walked, since the `IsProviderFor` bound just
/// re-states the provider's `where` clause, which the real provider impl already carries.
pub fn unmet_clauses<'tcx>(
    tcx: TyCtxt<'tcx>,
    failing: &[ty::PolyTraitClause<'tcx>],
) -> Vec<ty::PolyTraitClause<'tcx>> {
    failing
        .iter()
        .copied()
        .filter(|nested| !is_workaround_plumbing(tcx, *nested))
        .collect()
}

/// What a node that is not descendable does with its `unmet` clauses: it follows the ones below, or
/// ends as a terminal leaf.
///
/// A foreign-type bound is normally the terminal root cause, and the descent must not wander
/// into whatever `std` blanket impl happens to satisfy it. Two exceptions are followed: a CGP
/// getter or blanket trait on a non-context type whose blanket impl depends on the *context*
/// (so the real cause surfaces and de-duplicates), and a same-trait recursion over a
/// type-level list (a record's `Cons<..>: HandleMapEntry<..>` whose tail is another
/// `Cons<.., Nil>: …`). A foreign `f64: FnPtr` step is neither, so the bound stays the leaf,
/// and so is a blanket impl that only reaches another context's provider: a dispatch payload
/// missing its own impl of the dispatched trait is the cause, not the enum-matching blanket
/// its type cannot use.
pub fn followable_children<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    context: Ty<'tcx>,
    self_fp: Fingerprint,
    unmet: Vec<ty::PolyTraitClause<'tcx>>,
) -> NodeStep<'tcx> {
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
        let leaf_ref = erased.skip_binder().trait_ref;
        return NodeStep::Terminal(terminal_result(
            tcx, erased, leaf_ref, parent, context, self_fp,
        ));
    }
    NodeStep::Descend(followable)
}

/// The result of a node that matched an impl whose trait clauses left nothing to walk. `failing` is
/// every clause that does not hold, all of them scaffolding.
pub fn result_without_unmet_clause<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
    failing: &[ty::PolyTraitClause<'tcx>],
    self_fp: Fingerprint,
) -> SubResult {
    // Only scaffolding fails: the context's own `IsProviderFor` beside its delegate, which a
    // hand-written `DelegateComponent` impl leaves out.
    if let Some(leaf) = wiring_without_provider_for(tcx, failing, context) {
        return cause_result(tcx, erased, context, leaf, self_fp);
    }
    // Matched an impl, yet every trait-clause `where`-obligation holds: the fault is a projection
    // the trait-clause walk cannot see: a `HasField::Value` mismatch (a field present with the
    // wrong type), or any other associated type the owner supplies differently from what the impl
    // requires (a CGP abstract type, typically). Either way the node matched an impl, so it is a
    // complete non-terminal and is cached.
    let leaf_ref = erased.skip_binder().trait_ref;
    match projection_mismatch(tcx, erased) {
        Some(mismatch) => projection_result(tcx, erased, leaf_ref, context, mismatch, self_fp),
        None => SubResult::empty(self_fp),
    }
}
