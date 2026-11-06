//! The walk's entry points, and the fold of the root node's sub-result into a `Resolved`.

use cargo_cgp_error_processing::{Causes, Resolved, ResolvedClass, WiringOverflow};
use rustc_middle::ty::{self, Ty, TypeVisitableExt as _};

use crate::resolve::ctx::ResolveCtx;
use crate::resolve::label::{render_ty, trait_generics};
use crate::resolve::walk::resolve_node;

/// Walk the dependency graph of `top`, the **real consumer-trait obligation** `Ctx:
/// ConsumerTrait<Params…>` the failure stands for, not a `CanUseComponent` wrapper, and, for each
/// distinct terminal unmet bound it bottoms out on, return that leaf with its rendered dependency
/// chain. The walk descends the consumer trait to its provider trait and on to the provider's real
/// `where` bounds, never through `IsProviderFor`. `None` when no branch reaches a resolvable leaf.
///
/// The descent is memoized at **every node** through the context's cache: one CGP mistake surfaces
/// the same failure at many sites (all seeding the same obligation), and a shared trait is a
/// diamond reached from several parents, so each distinct obligation is resolved once and reused.
/// See `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.
pub fn resolve_leaves<'tcx>(
    cx: ResolveCtx<'_, 'tcx>,
    top: ty::PolyTraitClause<'tcx>,
) -> Option<Resolved> {
    let tcx = cx.tcx;
    // Erase the seed's free regions up front, exactly as every descendant obligation is erased by
    // [`impl_where_obligations`]. A lifetime-parameterized check entry (`(Life<'a>, str)`) seeds an
    // obligation carrying the check impl's own `'a`, and leaving it unerased would make the seed
    // the one chain entry whose `self_ty == context` comparisons fail, mislabeling the consumer
    // node.
    let top = tcx.erase_and_anonymize_regions(top);
    let context = top.skip_binder().self_ty();
    compute_leaves(cx, top, context)
}

/// Walk the dependency graph of `top`, the provider obligation `Provider: ProviderTrait<Ctx, …>` a
/// `#[check_providers]` entry or a call to the provider's own method asserts, with `context` as the
/// walk's root context. The result is the provider class: its trait is the provider trait (named
/// without the provider and the context), and its class holds that provider. `None` when no branch
/// reaches a resolvable leaf.
pub fn resolve_provider_leaves<'tcx>(
    cx: ResolveCtx<'_, 'tcx>,
    top: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> Option<Resolved> {
    let tcx = cx.tcx;
    let top = tcx.erase_and_anonymize_regions(top);
    let context = tcx.erase_and_anonymize_regions(context);
    let mut resolved = compute_leaves(cx, top, context)?;
    let top_ref = top.skip_binder().trait_ref;
    resolved.consumers = vec![format!(
        "{}{}",
        tcx.item_name(top_ref.def_id),
        trait_generics(tcx, top_ref, 2)
    )];
    resolved.class = ResolvedClass::Provider(render_ty(tcx, top_ref.self_ty()));
    Some(resolved)
}

/// Why walking `top` was cut short, when a guard cut it: the obligation met again (a cycle in the
/// wiring) or the depth cap reached without a repeat (a lookup that grows). `None` when the walk
/// ran to its leaves, as it does through a type the compiler could not resolve, and for a seed
/// holding a type parameter.
pub fn walk_overflow<'tcx>(
    cx: ResolveCtx<'_, 'tcx>,
    top: ty::PolyTraitClause<'tcx>,
) -> Option<WiringOverflow> {
    let tcx = cx.tcx;
    // As in [`compute_leaves`]: under an empty parameter environment the walk cannot read the
    // wiring of a seed holding a type parameter, so a cut would name a provider it never found.
    if top.has_param() {
        return None;
    }
    let top = tcx.erase_and_anonymize_regions(top);
    let context = top.skip_binder().self_ty();
    resolve_node(cx, top, None, context, &[], 0).cut
}

/// Fold the root node's owned sub-causes into a [`Resolved`]: group by leaf into one
/// [`Cause`](cargo_cgp_error_processing::Cause) per distinct root cause, each holding every path
/// that reaches it (so a shared trait's diamond survives to the renderer). Merging happens in the
/// rustc-free [dependency graph](cargo_cgp_error_processing::DependencyGraph) at render time, not
/// here. `top` is already region-erased and `context` is its self type. `None` for a seed holding a
/// type parameter, as well as for a walk that reaches no resolvable leaf.
pub fn compute_leaves<'tcx>(
    cx: ResolveCtx<'_, 'tcx>,
    top: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> Option<Resolved> {
    let tcx = cx.tcx;
    // The walk solves under an empty parameter environment, where the solver does not prove a
    // wiring lookup that names the seed's type parameter, so the walk would blame wiring that
    // exists.
    if top.has_param() {
        return None;
    }
    let sub = resolve_node(cx, top, None, context, &[], 0);

    // One cause per distinct leaf; a leaf reached by several paths keeps each path, so the graph
    // can render the convergence rather than dropping every path but the first.
    let causes = Causes::from_sub_chains(sub.causes.into_iter().map(|sc| (sc.leaf, sc.path)));

    if causes.is_empty() {
        return None;
    }

    // The consumer trait is the seed obligation's own trait, named straight off its `DefId` with
    // its parameters reattached from the obligation's arguments, with no marker, no name map, no
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
        class: ResolvedClass::Consumer,
        // The subject is the checked context itself.
        subject_is_context: true,
        causes,
    })
}
