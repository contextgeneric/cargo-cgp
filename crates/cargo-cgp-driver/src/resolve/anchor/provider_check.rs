//! The `#[check_providers]` anchor: matching the failing provider entry by span.

use cargo_cgp_error_processing::Resolved;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::{CGP_COMPONENT_CRATE, IS_PROVIDER_FOR_TRAIT};
use crate::resolve::anchor::provider_obligation;
use crate::resolve::ctx::ResolveCtx;
use crate::resolve::query::{check_impls_at, is_cgp_item, marker_to_consumer};
use crate::resolve::walk::resolve_provider_leaves;

/// Resolve the root cause(s) of the `#[check_providers]` entry whose diagnostic caret sits at
/// `primary_span`, or `None` if no such entry is there.
///
/// `check_components!` lowers a `#[check_providers(P)]` entry to an impl of a generated check trait
/// whose supertrait is `IsProviderFor<Component, Context, Params>`, with the listed provider as
/// `Self` and its span on the entry the user wrote. The anchor reads that assertion only to learn
/// which provider, component, and context the entry names, then walks the real provider obligation
/// `P: ProviderTrait<Context, Params…>`, so the tree starts at the provider the entry lists rather
/// than at the context's own wiring, which may not wire that provider at all.
pub fn resolve_provider_check(cx: ResolveCtx<'_, '_>, primary_span: Span) -> Option<Resolved> {
    let tcx = cx.tcx;
    for check in check_impls_at(tcx, primary_span, provider_check_supertrait) {
        let is_provider_for = check.asserted.as_trait_clause()?.skip_binder().trait_ref;
        // `IsProviderFor<Component, Context, Params>` on the provider: args are
        // `[Provider, Component, Context, Params]`.
        let provider = is_provider_for.self_ty();
        let marker = is_provider_for.args.type_at(1);
        let context = is_provider_for.args.type_at(2);
        let params = is_provider_for.args.type_at(3);
        let (consumer_did, provider_did) = marker_to_consumer(tcx, marker)?;
        let top = provider_obligation(tcx, provider, provider_did, context, consumer_did, params)?;
        if let Some(resolved) = resolve_provider_leaves(cx, top, context) {
            return Some(resolved);
        }
    }
    None
}

/// The `IsProviderFor<..>` supertrait clause of `trait_did` when it is a provider check trait: one
/// generic over the component it checks, as `#[check_providers]` generates. A component's own
/// provider trait also carries `IsProviderFor` as a supertrait, but keyed on its own marker, so it
/// never matches. Anchored by `DefId` to `cgp_component`.
fn provider_check_supertrait(tcx: TyCtxt<'_>, trait_did: DefId) -> Option<ty::Clause<'_>> {
    for &(clause, _) in tcx.explicit_super_clauses_of(trait_did).skip_binder() {
        if let Some(tp) = clause.as_trait_clause()
            && is_cgp_item(tcx, tp.def_id(), IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
            && matches!(
                tp.skip_binder().trait_ref.args.type_at(1).kind(),
                ty::Param(_)
            )
        {
            return Some(clause);
        }
    }
    None
}
