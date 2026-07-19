//! Which obligations the descent walks into, and which it drops as scaffolding.

use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE, DELEGATE_COMPONENT_TRAIT,
    HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
};
use crate::resolve::cgp_item::{is_cgp_item, is_provider_trait};

/// Whether `pred` is the check-trait scaffolding — `CanUseComponent` or `IsProviderFor` — the walk
/// resolves *around* rather than through. These sit beside the real consumer/provider-trait
/// obligation in every generated blanket impl and carry only the delegation check
/// (`CanUseComponent`) or a copy of the provider's `where` clause (`IsProviderFor`), both redundant
/// with the real obligation the walk follows instead. Dropping them as dependencies is what keeps
/// the resolution independent of `IsProviderFor`.
pub(crate) fn is_workaround_plumbing<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
) -> bool {
    let did = pred.skip_binder().trait_ref.def_id;
    is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
}

/// Whether the descent should walk *into* `pred`'s dependencies, rather than treat `pred` as a
/// terminal leaf. It descends any **provider trait** (a `ProvideFoo: Foo<App>` bound routes on to
/// the provider's own real `where` bounds), the `DelegateComponent` table lookup, and any
/// obligation on the context itself (its consumer, getter, and capability traits). It stops at
/// everything else — an ordinary bound like `f64: Eq`, whose `Self` is a foreign type, is a leaf.
///
/// It deliberately does *not* descend `CanUseComponent`/`IsProviderFor`: the resolver reads a
/// provider's bounds from the provider trait's own impl, not from the `IsProviderFor` marker's
/// copy of them, so those obligations are dropped as [workaround plumbing](is_workaround_plumbing)
/// while the real provider-trait obligation beside them carries the cause.
pub(crate) fn is_descendable<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
) -> bool {
    let trait_ref = pred.skip_binder().trait_ref;
    let did = trait_ref.def_id;
    tcx.erase_and_anonymize_regions(trait_ref.self_ty()) == context
        || is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_provider_trait(tcx, did)
}

/// Whether a trait predicate is a genuine CGP `HasField` bound — the missing-field leaf.
pub(crate) fn is_has_field(tcx: TyCtxt<'_>, pred: ty::PolyTraitPredicate<'_>) -> bool {
    is_cgp_item(
        tcx,
        pred.skip_binder().def_id(),
        HAS_FIELD_TRAIT,
        CGP_FIELD_CRATE,
    )
}
