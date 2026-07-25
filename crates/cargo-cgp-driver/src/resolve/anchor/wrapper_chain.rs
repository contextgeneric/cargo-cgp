//! The foreign-wrapper anchor: descending a `where`-clause chain to a CGP consumer.

use cargo_cgp_error_processing::{Cause, ChainNode, DepNode, Resolved, prepend_hop};
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_middle::ty::print::PrintTraitRefExt as _;
use rustc_middle::ty::{
    self, TyCtxt, TypeVisitableExt as _, TypingMode, Unnormalized, Upcast as _,
};
use rustc_span::{DUMMY_SP, Span};
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::anchor::enclosing_trait_impls;
use crate::resolve::cache::ResolveCache;
use crate::resolve::cgp_item::{consumer_provider_trait, is_local_adt, is_provider_trait};
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve the root cause(s) of a CGP wiring failure reported *inside a hand-written `impl Trait for
/// Foreign` block whose `Self` is a foreign type holding the context* — the transfer example's
/// `impl CanAddApiRoutes for Router<Arc<MockApp>>`, where the routing wrapper's supertrait descends
/// through a chain of ordinary user-trait `where`-clauses (`… CanAddRoute<MockApp, …>` → `MockApp:
/// CanHandleApiSend<…>`) before it reaches a CGP consumer on the context. The context appears only
/// as a type *argument* of the failing traits, never as the impl's `Self`, so
/// [`resolve_impl_site`](super::resolve_impl_site)'s "direct supertrait on a local context" recovery
/// cannot fire.
///
/// This entry starts from the enclosing impl's own unmet supertrait and walks *down* the ordinary
/// trait obligations — via each impl's `where`-clauses — until one lands on a CGP consumer whose
/// `Self` *is* a local context, at which point it hands off to [`consumer_handoff_causes`]. Every
/// ordinary hop between the impl and that handoff becomes a `trait impl` node, so the tree reads
/// from the code the programmer wrote (`CanAddApiRoutes → CanAddMainApiRoutes → CanAddRoute →
/// CanHandleApiSend → CanHandleApi → …`) down to the root cause. Because it *re-evaluates* each
/// obligation with the trait solver rather than trusting rustc's cascade-suppressed diagnostic, it
/// recovers the cause even where rustc's own error names only the outermost unsatisfied bound.
/// `None` when no enclosing impl's supertrait chain reaches a CGP consumer on a local context.
pub fn resolve_wrapper_chain(
    tcx: TyCtxt<'_>,
    cache: &ResolveCache,
    spans: &[Span],
) -> Option<Resolved> {
    for impl_did in enclosing_trait_impls(tcx, spans) {
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        // A provider-trait impl's `Self` is a provider struct, and its supertrait is `IsProviderFor`,
        // so descending it would route the cause through that workaround — leaking `IsProviderFor` and
        // the provider trait's own `__Context__` parameter into the tree. A caret on a provider's own
        // impl is a documented decline (mirrored in [`resolve_impl_site`](super::resolve_impl_site)).
        if is_provider_trait(tcx, trait_ref.def_id) {
            continue;
        }
        let self_ty = tcx.erase_and_anonymize_regions(trait_ref.self_ty());
        // The direct-supertrait, local-`Self` case is `resolve_impl_site`'s (tried first); here we
        // handle the rest, where the CGP consumer is reached only through `where`-clause hops.
        let wrapper = trait_ref.print_only_trait_path().to_string();
        let top_node = DepNode::Trait {
            trait_ref: wrapper.clone(),
            self_ty: self_ty.to_string(),
        };

        // Descend each unmet supertrait of the impl's own trait, collecting a cause per CGP handoff
        // reached beneath it.
        let mut causes: Vec<Cause> = Vec::new();
        for &(clause, _) in tcx
            .explicit_super_predicates_of(trait_ref.def_id)
            .skip_binder()
        {
            let concrete = clause.instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref));
            let Some(sup) = concrete.as_trait_clause() else {
                continue;
            };
            if holds(tcx, sup) {
                continue;
            }
            collect_wrapper_chain_causes(tcx, cache, sup, &[], 0, &mut causes);
        }

        if !causes.is_empty() {
            // Head every cause's paths with the impl's own trait — the code the programmer wrote —
            // and keep one cause per distinct leaf, since separate supertraits can descend to the
            // same one.
            let causes = prepend_hop(&causes, &top_node);
            return Some(Resolved {
                context: self_ty.to_string(),
                consumers: vec![wrapper],
                // The impl's own trait heads the header; a routing wrapper such as `CanAddApiRoutes`
                // is a plain trait, not a CGP consumer, so it reads `the trait` (`CGP-E009`).
                consumers_are_cgp: consumer_provider_trait(tcx, trait_ref.def_id).is_some(),
                // `Self` is a foreign wrapper holding the context, not the context itself — reached
                // here only *because* it is not the context (else `resolve_impl_site` caught it).
                subject_is_context: false,
                causes,
            });
        }
    }
    None
}

/// Bound on how deep the wrapper-chain descent walks before giving up, matching the dependency-graph
/// walk's own bound. Real wrapper chains (`CanAddApiRoutes → CanAddMainApiRoutes → CanAddRoute → …`)
/// are only a few hops.
const MAX_WRAPPER_DEPTH: u32 = 32;

/// Descend `obligation` — an unmet ordinary-trait bound — looking for the nearest CGP consumer on a
/// local context beneath it, and append a cause per one found to `out`. Such a consumer obligation
/// (`App: CanHandleApi<GreetApi>`) is the handoff: [`consumer_handoff_causes`] recovers its
/// dependency tree, the descent stops there (that recovery walks the CGP chain the rest of the way),
/// and the ordinary hops that led here are prepended as `trait impl` nodes. Anything else is
/// descended through its satisfying impl's `where`-clause obligations ([`wrapper_chain_children`]),
/// following only the unmet ones.
///
/// The descent reaches the consumer two ways, both handled by `wrapper_chain_children`: as a direct
/// `where`-clause bound, or — the transfer example's case — as the *base* of a projection bound
/// (`<App as CanHandleApi<GreetApi>>::Response: Send`), since a `where`-clause naming an associated
/// type of the broken consumer is what makes the outer trait genuinely fail (the direct
/// `App: CanHandleApiSend<GreetApi>` bound is instead *assumed to hold* off its ill-formed impl, so
/// it is never the failing route). A branch with no satisfying impl, or one reaching no consumer
/// within [`MAX_WRAPPER_DEPTH`], contributes nothing — only a genuine CGP consumer is ever reported,
/// so a wandering descent cannot fabricate a cause.
fn collect_wrapper_chain_causes<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    obligation: ty::PolyTraitPredicate<'tcx>,
    chain: &[DepNode],
    depth: u32,
    out: &mut Vec<Cause>,
) {
    if depth > MAX_WRAPPER_DEPTH {
        return;
    }

    // The handoff: `obligation` is a CGP consumer on a local context. Recover its cause tree and
    // prepend the chain of ordinary hops that led here.
    if let Some(causes) = consumer_handoff_causes(tcx, cache, obligation) {
        // Prepend the ordinary hops that led here to each path. The caller merges `out` by leaf, so
        // alternative routes reaching one cause down different branches of the chain all survive.
        for cause in causes {
            out.push(Cause {
                leaf: cause.leaf,
                paths: cause
                    .paths
                    .into_iter()
                    .map(|path| prepend_chain(chain, path))
                    .collect(),
            });
        }
        return;
    }

    // Not a handoff: descend the obligations of the impl that would satisfy it — its direct trait
    // bounds and the base trait ref of each associated-type `where`-clause bound.
    let Some(children) = wrapper_chain_children(tcx, obligation) else {
        return;
    };
    let trait_ref = obligation.skip_binder().trait_ref;
    let node = DepNode::Trait {
        trait_ref: trait_ref.print_only_trait_path().to_string(),
        self_ty: tcx
            .erase_and_anonymize_regions(trait_ref.self_ty())
            .to_string(),
    };
    let mut next_chain = chain.to_vec();
    next_chain.push(node);
    for child in children {
        if holds(tcx, child) {
            continue;
        }
        collect_wrapper_chain_causes(tcx, cache, child, &next_chain, depth + 1, out);
    }
}

/// Recover a cause per root cause of `obligation` when it *is* a CGP consumer trait on a local
/// context (`App: CanHandleApi<GreetApi>`) — the handoff the wrapper-chain descent hands off to the
/// ordinary [walk](crate::resolve::walk). `obligation` already is the consumer obligation the walk
/// wants — with its concrete component parameters preserved — so it is walked directly, with no
/// `CanUseComponent`/`IsProviderFor` detour, and the returned trees are already headed by the
/// consumer node. `None` when `obligation`'s `Self` is not a local ADT or its trait is not a CGP
/// consumer — so the descent keeps walking rather than stopping on an ordinary bound.
fn consumer_handoff_causes<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    obligation: ty::PolyTraitPredicate<'tcx>,
) -> Option<Vec<Cause>> {
    let trait_ref = obligation.skip_binder().trait_ref;
    let context = tcx.erase_and_anonymize_regions(trait_ref.self_ty());
    if !is_local_adt(context) {
        return None;
    }
    // A CGP consumer trait pairs with a provider trait through its blanket impl; a plain trait does
    // not, so it is not a handoff.
    consumer_provider_trait(tcx, trait_ref.def_id)?;
    let resolved = resolve_leaves(tcx, cache, obligation)?;
    Some(resolved.causes)
}

/// The obligations to descend from an unmet ordinary-trait bound: the `where`-clause obligations of
/// the impl that would satisfy it, with each associated-type bound replaced by its *base* trait ref
/// (`<App as CanHandleApi<Api>>::Response: Send` → `App: CanHandleApi<Api>`). The base is what a
/// projection bound really rests on, and it is concrete even when the projected type itself is not —
/// which is exactly why the ordinary
/// [`impl_where_obligations`](crate::resolve::walk::impl_where_obligations) (which normalizes,
/// turning the projection into an inference variable it then drops) cannot surface it. `None` when
/// no impl matches. Mirrors `impl_where_obligations`'s next-solver-safe impl match, but reads the
/// impl's predicates un-normalized so an associated-type `Self` survives long enough to read its
/// base.
fn wrapper_chain_children<'tcx>(
    tcx: TyCtxt<'tcx>,
    obligation: ty::PolyTraitPredicate<'tcx>,
) -> Option<Vec<ty::PolyTraitPredicate<'tcx>>> {
    let param_env = ty::ParamEnv::empty();

    for impl_did in tcx.all_impls(obligation.def_id()) {
        let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
        let ocx = ObligationCtxt::new(&infcx);

        // Instantiate any higher-ranked binder with placeholders before relating, so a `for<'a>`
        // bound in the wrapper chain does not feed an escaping bound var into `ocx.eq` and panic
        // rustc's generalizer (see [`impl_where_obligations`](crate::resolve::walk)). A no-op for a
        // binder-free obligation.
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

        let mut children = Vec::new();
        for (predicate, _) in tcx.predicates_of(impl_did).instantiate(tcx, impl_args) {
            // Keep the predicate un-normalized so an associated-type `Self` survives; `skip_norm_wip`
            // unwraps without normalizing, unlike the `ocx.normalize` a trait-clause walk uses.
            let clause = infcx.resolve_vars_if_possible(predicate.skip_norm_wip());
            let clause = tcx.erase_and_anonymize_regions(clause);
            let Some(tp) = clause.as_trait_clause() else {
                continue;
            };
            // An associated-type bound (`<App as CanHandleApi<Api>>::Response: Send`) descends to the
            // projection's base trait (`App: CanHandleApi<Api>`) — what it truly rests on — rather
            // than the bound itself.
            let child = match tp.self_ty().skip_binder().kind() {
                ty::Alias(_, alias) if matches!(alias.kind, ty::AliasTyKind::Projection { .. }) => {
                    ty::Binder::dummy(alias.trait_ref(tcx)).upcast(tcx)
                }
                _ => tp,
            };
            // An unconstrained impl parameter leaves inference vars behind; such a child cannot be
            // re-checked in a fresh context, so skip it.
            if child.has_non_region_infer() {
                continue;
            }
            children.push(child);
        }
        return Some(children);
    }
    None
}

/// Prepend a top-to-bottom list of ordinary-hop nodes to `path`, so `chain[0]` becomes the outermost
/// node above the recovered CGP chain.
fn prepend_chain(chain: &[DepNode], mut path: Vec<ChainNode>) -> Vec<ChainNode> {
    for hop in chain.iter().rev() {
        path.insert(0, ChainNode::Hop(hop.clone()));
    }
    path
}
