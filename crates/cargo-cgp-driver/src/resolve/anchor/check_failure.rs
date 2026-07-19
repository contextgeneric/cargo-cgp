//! The `check_components!` anchor: matching the failing entry by span.

use cargo_cgp_error_processing::Resolved;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::{CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE};
use crate::resolve::anchor::{consumer_obligation, impl_self_ty_span};
use crate::resolve::cgp_item::{is_cgp_item, marker_to_consumer};
use crate::resolve::walk::resolve_leaves;

/// Resolve the root cause(s) of the check failure whose diagnostic caret sits at `primary_span`,
/// or `None` if this is not a resolvable `check_components!` failure (in which case the caller
/// leaves the original diagnostic to the text-rewrite fallback).
///
/// The check impl's supertrait is the user's own `CanUseComponent<Marker, Params>` assertion, which
/// this reads to learn *which* component the entry checks — but it then walks the real consumer
/// obligation `Ctx: ConsumerTrait<Params…>` that marker stands for, never the `CanUseComponent` /
/// `IsProviderFor` scaffolding, so the resolution does not depend on `IsProviderFor`.
pub fn resolve_check_failure(tcx: TyCtxt<'_>, primary_span: Span) -> Option<Resolved> {
    for trait_did in tcx.all_traits_including_private() {
        let Some(super_clause) = can_use_component_supertrait(tcx, trait_did) else {
            continue;
        };

        for impl_did in tcx.all_impls(trait_did) {
            // The entry the error is about is the impl whose `Self` type carries the caret's
            // span (the macro re-spans the context type onto the entry, so they coincide).
            if impl_self_ty_span(tcx, impl_did) != Some(primary_span) {
                continue;
            }

            let trait_ref = tcx
                .impl_trait_ref(impl_did)
                .instantiate_identity()
                .skip_norm_wip();
            let concrete = super_clause.instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref));

            let Some(can_use) = concrete.as_trait_clause() else {
                continue;
            };
            let Some(top) = can_use_to_consumer_obligation(tcx, can_use) else {
                continue;
            };
            if let Some(resolved) = resolve_leaves(tcx, top) {
                return Some(resolved);
            }
        }
    }
    None
}

/// Turn a `Ctx: CanUseComponent<Marker, Params>` assertion into the real consumer obligation
/// `Ctx: ConsumerTrait<Params…>` it stands for: the marker is mapped to its consumer trait
/// (`IsProviderFor`-free, via [`marker_to_consumer`]) and the `Params` slot is ungrouped back into
/// the consumer's own arguments. This is what lets the check and use-site anchors feed the walk a
/// real consumer obligation instead of the `CanUseComponent` wrapper. `None` when the marker keys
/// no known consumer trait, or the slot does not match the consumer's parameters.
fn can_use_to_consumer_obligation<'tcx>(
    tcx: TyCtxt<'tcx>,
    can_use: ty::PolyTraitPredicate<'tcx>,
) -> Option<ty::PolyTraitPredicate<'tcx>> {
    let trait_ref = can_use.skip_binder().trait_ref;
    // `CanUseComponent<Marker, Params>` — args are `[Ctx, Marker, Params]`.
    let context = trait_ref.self_ty();
    let marker = trait_ref.args.type_at(1);
    let params = trait_ref.args.type_at(2);
    let (consumer_did, _) = marker_to_consumer(tcx, marker)?;
    consumer_obligation(tcx, context, consumer_did, params)
}

/// The `CanUseComponent<..>` supertrait clause of `trait_did`, if it carries one — the marker
/// of a `check_components!` check trait. Anchored by DefId to `cgp_component`.
fn can_use_component_supertrait(tcx: TyCtxt<'_>, trait_did: DefId) -> Option<ty::Clause<'_>> {
    for &(clause, _) in tcx.explicit_super_predicates_of(trait_did).skip_binder() {
        if let Some(tp) = clause.as_trait_clause()
            && is_cgp_item(
                tcx,
                tp.def_id(),
                CAN_USE_COMPONENT_TRAIT,
                CGP_COMPONENT_CRATE,
            )
        {
            return Some(clause);
        }
    }
    None
}
