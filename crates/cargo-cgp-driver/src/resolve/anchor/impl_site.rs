//! The impl-site anchor: a wiring failure surfaced inside a hand-written `impl Trait for Context`.

use cargo_cgp_error_processing::{Cause, DepNode, Resolved, prepend_hop};
use rustc_middle::ty::print::PrintTraitRefExt as _;
use rustc_middle::ty::{self, TyCtxt, Upcast as _};
use rustc_span::Span;

use crate::resolve::anchor::enclosing_trait_impls;
use crate::resolve::cache::ResolveCache;
use crate::resolve::cgp_item::{
    consumer_provider_trait, is_capability_trait, is_local_adt, is_provider_trait,
};
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve the root cause(s) of a CGP wiring failure reported *inside a hand-written `impl Trait
/// for Context` block* — the shape a wrapper trait that carries a CGP consumer trait as a
/// supertrait produces when it is implemented directly on a concrete context (the transfer
/// example's per-endpoint `impl CanHandleApiSend<Api> for MockApp`, added to bound a future
/// `Send`). Such a failure anchors on neither a `check_components!` entry nor a consumer-method
/// call, and its caret sits on the impl rather than on the context's own type definition, so
/// [`resolve_use_site`](super::resolve_use_site) cannot recover the context from a
/// struct-definition span.
///
/// This entry recovers the context from the enclosing impl's `Self` type, and — crucially — the
/// *exact* failing obligation from the impl's CGP consumer supertrait, so a generic component
/// carries its concrete parameter (`CanCalculateArea<Rectangle>`, not the `()` form the use-site
/// re-check would substitute). It reconstructs the `Ctx: CanUseComponent<Marker, Params>`
/// obligation that supertrait stands for and walks it exactly as a check entry would. The recovered
/// tree is then headed by the impl's *own* trait — the wrapper the programmer wrote — so the
/// diagnostic points at their code and the CGP consumer it reduces to follows beneath: the failure
/// reads `CanHandleApiSend → CanHandleApi → …`, not `CanHandleApi → …` with the wrapper dropped.
/// Because the wrapper is a distinct trait from that supertrait, its error stands on its own rather
/// than de-duplicating into the `check_components!` entry. `None` when no enclosing impl on a local
/// context carries an unmet, reconstructable CGP consumer supertrait.
pub fn resolve_impl_site(
    tcx: TyCtxt<'_>,
    cache: &ResolveCache,
    spans: &[Span],
) -> Option<Resolved> {
    for impl_did in enclosing_trait_impls(tcx, spans) {
        // Safe because `enclosing_trait_impls` keeps only `of_trait` impls.
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        // A provider-trait impl (`impl Runner<Ctx> for RunViaInner`) has a *provider* struct as its
        // `Self`, not a context, and its only supertrait is `IsProviderFor` — so the supertrait
        // recovery below would reach a consumer only via the `IsProviderFor` workaround this resolver
        // sheds (leaking `IsProviderFor` and the trait's `__Context__` into the tree). Skip that. But
        // the provider's own `where` clause can name a cross-context dependency on a *concrete* local
        // context (`where Inner: CanCompute`); such a bound failing is a real wiring failure, so
        // recover it as the consumer obligation it is — de-duplicating into that context's own check.
        if is_provider_trait(tcx, trait_ref.def_id) {
            if let Some(resolved) = cross_context_where_bound(tcx, cache, impl_did) {
                return Some(resolved);
            }
            continue;
        }
        let context = tcx.erase_and_anonymize_regions(trait_ref.self_ty());
        // Only a local struct/enum is a context whose wiring we can re-check; skip an impl on a
        // foreign type (e.g. `impl … for Router<Arc<App>>`) or a type parameter. Such a foreign
        // wrapper — where the CGP consumer failure sits several `where`-clause hops down rather than
        // in a direct supertrait — is [`resolve_wrapper_chain`](super::resolve_wrapper_chain)'s job.
        if !is_local_adt(context) {
            continue;
        }

        // The error the programmer actually wrote is the impl's *own* trait — the wrapper (e.g.
        // `CanHandleApiSend<Api>`), not the CGP consumer supertrait it reduces to. That original
        // obligation heads the dependency tree and names the diagnostic, so the reader sees the
        // failure at their own code; the CGP consumer it depends on follows as the next node. Being
        // a distinct trait from that supertrait, the wrapper's error is reported on its own rather
        // than de-duplicated into the `check_components!` entry for the supertrait.
        let obligation: ty::PolyTraitPredicate<'_> = ty::Binder::dummy(trait_ref).upcast(tcx);
        if let Some((consumers_are_cgp, causes)) = wrapper_consumer_causes(tcx, cache, obligation) {
            return Some(Resolved {
                context: context.to_string(),
                consumers: vec![trait_ref.print_only_trait_path().to_string()],
                consumers_are_cgp,
                // The wrapper is implemented directly on the context, so the subject is the context.
                subject_is_context: true,
                causes,
            });
        }
    }
    None
}

/// Recover a cross-context dependency named directly in a provider impl's own `where` clause: an
/// unmet `Ctx: Consumer` bound whose `Ctx` is a *concrete* local context and whose trait is a CGP
/// consumer (`where Inner: CanCompute`). Such a bound is a real wiring failure the provider imposes
/// on another context; walked as the consumer obligation it is, it yields that context's own
/// root-cause tree — the same `Resolved` the context's `check_components!` entry produces, so the two
/// de-duplicate. `None` when the impl's `where` clause carries no such bound. A normal provider's
/// dependencies bind the *generic* context (`Ctx: HasName`, `Self` a type parameter), whose `self_ty`
/// is not a local ADT, so this fires only on the cross-context shape.
fn cross_context_where_bound<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    impl_did: rustc_span::def_id::DefId,
) -> Option<Resolved> {
    for &(clause, _) in tcx.predicates_of(impl_did).predicates {
        let Some(bound) = clause.as_trait_clause() else {
            continue;
        };
        let trait_ref = bound.skip_binder().trait_ref;
        if !is_local_adt(trait_ref.self_ty()) {
            continue;
        }
        if consumer_provider_trait(tcx, trait_ref.def_id).is_none() {
            continue;
        }
        if holds(tcx, bound) {
            continue;
        }
        if let Some(resolved) = resolve_leaves(tcx, cache, bound) {
            return Some(resolved);
        }
    }
    None
}

/// Recover a cause per unmet CGP consumer supertrait of a wrapper obligation `Self: Wrapper` whose
/// `Self` is a local context, each tree headed by the wrapper's own `trait impl` node. Returns
/// whether the wrapper is itself a CGP consumer (for the header wording) and the causes, or `None`
/// when `Self` is not a local ADT or no CGP consumer supertrait is unmet — so a plain bound like
/// `Send`, or a wrapper on a foreign type, is skipped rather than mis-recovered.
fn wrapper_consumer_causes<'tcx>(
    tcx: TyCtxt<'tcx>,
    cache: &ResolveCache,
    obligation: ty::PolyTraitPredicate<'tcx>,
) -> Option<(bool, Vec<Cause>)> {
    let trait_ref = obligation.skip_binder().trait_ref;
    let context = tcx.erase_and_anonymize_regions(trait_ref.self_ty());
    if !is_local_adt(context) {
        return None;
    }
    let wrapper = trait_ref.print_only_trait_path().to_string();
    let wrapper_node = DepNode::Trait {
        trait_ref: wrapper,
        self_ty: context.to_string(),
    };

    let mut causes: Vec<Cause> = Vec::new();
    // Each supertrait the wrapper trait carries, instantiated for this `Self`. A CGP consumer trait
    // among them that does not hold is the wiring failure the wrapper surfaces.
    for &(clause, _) in tcx
        .explicit_super_predicates_of(trait_ref.def_id)
        .skip_binder()
    {
        let concrete = clause.instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref));
        let Some(sup) = concrete.as_trait_clause() else {
            continue;
        };
        if tcx.erase_and_anonymize_regions(sup.skip_binder().self_ty()) != context {
            continue;
        }
        if holds(tcx, sup) {
            continue;
        }
        // Only a CGP capability supertrait is the wiring failure the wrapper surfaces; a plain
        // supertrait such as `Send` is not. That is either a CGP *component* consumer trait, or a
        // `#[cgp_fn]` / `#[blanket_trait]` blanket-impl trait (`impl<Context> Trait for Context
        // where Self: HasField<…>`), whose failing bound has a recoverable cause down the blanket's
        // `where` clause. Walk the supertrait obligation directly in either case.
        let sup_did = sup.skip_binder().trait_ref.def_id;
        if consumer_provider_trait(tcx, sup_did).is_none() && !is_capability_trait(tcx, sup_did) {
            continue;
        }
        let Some(resolved) = resolve_leaves(tcx, cache, sup) else {
            continue;
        };
        causes.extend(resolved.causes);
    }

    if causes.is_empty() {
        return None;
    }
    // Head each recovered path with the wrapper hop, so the CGP chain hangs beneath the trait the
    // programmer wrote, and keep one cause per distinct leaf — so alternative paths to one cause
    // survive rather than only the first supertrait's.
    let causes = prepend_hop(&causes, &wrapper_node);
    // Whether the wrapper trait is a CGP *consumer* trait or a plain wrapper, decided by its
    // fingerprint: a consumer trait carries a blanket impl routing to a provider trait
    // (`consumer_provider_trait`), while a hand-written wrapper like `CanHandleApiSend` has only its
    // concrete impl. This picks `the consumer trait` vs `the trait` in the header.
    let consumers_are_cgp = consumer_provider_trait(tcx, trait_ref.def_id).is_some();
    Some((consumers_are_cgp, causes))
}
