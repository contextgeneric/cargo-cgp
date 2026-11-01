//! Which obligations the descent walks into, and which it drops as scaffolding.

use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE, DELEGATE_COMPONENT_TRAIT,
    HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
};
use crate::resolve::data::is_data_trait;
use crate::resolve::query::{is_cgp_item, is_provider_trait};

/// Whether `pred` is the check-trait scaffolding (`CanUseComponent` or `IsProviderFor`), the walk
/// resolves *around* rather than through. These sit beside the real consumer/provider-trait
/// obligation in every generated blanket impl and carry only the delegation check
/// (`CanUseComponent`) or a copy of the provider's `where` clause (`IsProviderFor`), both redundant
/// with the real obligation the walk follows instead. Dropping them as dependencies is what keeps
/// the resolution independent of `IsProviderFor`.
pub(crate) fn is_workaround_plumbing<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
) -> bool {
    let did = pred.skip_binder().trait_ref.def_id;
    is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
}

/// Whether the descent should walk *into* `pred`'s dependencies, rather than treat `pred` as a
/// terminal leaf: a [context-side](is_context_side) obligation, or one of CGP's
/// **extensible-data traits** (a cast, a builder or extractor operation), whose blanket impl leads
/// to the presence-marker bound that names the field or variant at fault. It stops at everything
/// else: an ordinary bound like `f64: Eq`, whose `Self` is a foreign type, is a leaf.
pub(crate) fn is_descendable<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> bool {
    is_context_side(tcx, pred, context) || is_data_trait(tcx, pred.skip_binder().def_id())
}

/// Whether `pred` belongs to the context's own wiring: any **provider trait** (a `ProvideFoo:
/// Foo<App>` bound routes on to the provider's own real `where` bounds), the `DelegateComponent`
/// table lookup, or any obligation on the context itself (its consumer, getter, and blanket
/// traits). A foreign bound the walk does not descend is still followed into these dependencies of
/// its blanket impl, since they reach the context, but not into a data trait's, which is about the
/// foreign type itself.
///
/// It deliberately does *not* cover `CanUseComponent`/`IsProviderFor`: the resolver reads a
/// provider's bounds from the provider trait's own impl, not from the `IsProviderFor` marker's
/// copy of them, so those obligations are dropped as [workaround plumbing](is_workaround_plumbing)
/// while the real provider-trait obligation beside them carries the cause.
pub(crate) fn is_context_side<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> bool {
    let trait_ref = pred.skip_binder().trait_ref;
    let did = trait_ref.def_id;
    tcx.erase_and_anonymize_regions(trait_ref.self_ty()) == context
        || is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_provider_trait(tcx, did)
}

/// Whether `pred` is a provider-trait obligation for a context other than `context`, such as a
/// dispatch matcher called with the unit context. Such a step belongs to another context's
/// wiring, so a non-context bound's blanket impl is not followed into it: that impl does not
/// depend on the context the walk is resolving. The context is the provider trait's first type
/// argument after `Self`, read by type position as the labels read it.
pub(crate) fn is_other_context_provider<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> bool {
    let trait_ref = pred.skip_binder().trait_ref;
    is_provider_trait(tcx, trait_ref.def_id)
        && trait_ref
            .args
            .types()
            .nth(1)
            .is_some_and(|provider_context| {
                tcx.erase_and_anonymize_regions(provider_context) != context
            })
}

/// Whether a trait predicate is a genuine CGP `HasField` bound: the missing-field leaf.
pub(crate) fn is_has_field(tcx: TyCtxt<'_>, pred: ty::PolyTraitClause<'_>) -> bool {
    is_cgp_item(
        tcx,
        pred.skip_binder().def_id(),
        HAS_FIELD_TRAIT,
        CGP_FIELD_CRATE,
    )
}
