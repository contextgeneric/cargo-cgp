//! Recognizing a component's traits and marker by the blanket impls `#[cgp_component]` generates.

use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::{CGP_COMPONENT_CRATE, CGP_TYPE_CRATE, DELEGATE_COMPONENT_TRAIT, USE_TYPE_TYPE};
use crate::resolve::query::is_cgp_item;

/// Whether `def_id` is a CGP *provider* trait, recognized **structurally** — by the provider
/// blanket impl `#[cgp_component]` generates, `impl<Ctx, P> ProviderTrait<Ctx> for P where P:
/// DelegateComponent<Marker>, …`, whose `Self` is a bare type parameter bounded by
/// `DelegateComponent`. This deliberately avoids the trait's `IsProviderFor` supertrait: the typed
/// resolver must not depend on `IsProviderFor`, which cargo-cgp aims to make obsolete, so it reads
/// the delegation-blanket shape instead — the same information without the workaround marker.
///
/// The descent treats a provider-trait bound as a step to walk into (its concrete impl carries the
/// provider's real `where` bounds), and a provider-trait obligation *for the context itself* is
/// routing that the tree drops.
pub fn is_provider_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    provider_blanket_marker(tcx, def_id).is_some()
}

/// The component marker a provider trait keys on, read from its provider blanket's `P:
/// DelegateComponent<Marker>` bound — the `IsProviderFor`-free replacement for reading the marker
/// off the `IsProviderFor<Marker, …>` supertrait. `None` when `def_id` has no such blanket (so it
/// is not a provider trait). Also serves as the provider-trait recognizer ([`is_provider_trait`]).
pub fn provider_blanket_marker<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> Option<Ty<'tcx>> {
    for &impl_did in tcx.trait_impls_of(def_id).blanket_impls() {
        let self_ty = tcx.type_of(impl_did).skip_binder();
        if !matches!(self_ty.kind(), ty::Param(_)) {
            continue;
        }
        for &(clause, _) in tcx.clauses_of(impl_did).clauses {
            let Some(tp) = clause.as_trait_clause() else {
                continue;
            };
            let trait_ref = tp.skip_binder().trait_ref;
            // The provider blanket bounds its own `Self` param on `DelegateComponent<Marker>`; the
            // marker is that bound's second argument.
            if trait_ref.self_ty() == self_ty
                && is_cgp_item(
                    tcx,
                    trait_ref.def_id,
                    DELEGATE_COMPONENT_TRAIT,
                    CGP_COMPONENT_CRATE,
                )
            {
                return trait_ref.args.get(1).and_then(|arg| arg.as_type());
            }
        }
    }
    None
}

/// The provider trait a consumer trait pairs with, found through the consumer's blanket impl
/// `impl<C> Consumer for C where C: Provider<C>`: the `where`-bound whose self type is the impl's
/// own self and whose trait is a [provider trait](is_provider_trait) is that provider. `None` when
/// `def_id` is not a CGP consumer trait — so [`is_consumer_trait`] is exactly this returning `Some`.
/// This is `IsProviderFor`-free: it reads only the consumer↔provider blanket link.
pub fn consumer_provider_trait(tcx: TyCtxt<'_>, consumer_did: DefId) -> Option<DefId> {
    for &impl_did in tcx.trait_impls_of(consumer_did).blanket_impls() {
        let impl_self = tcx.type_of(impl_did).skip_binder();
        for &(clause, _) in tcx.clauses_of(impl_did).clauses {
            let Some(tp) = clause.as_trait_clause() else {
                continue;
            };
            let trait_ref = tp.skip_binder().trait_ref;
            if trait_ref.self_ty() == impl_self && is_provider_trait(tcx, trait_ref.def_id) {
                return Some(trait_ref.def_id);
            }
        }
    }
    None
}

/// Whether `def_id` is a CGP *consumer* trait — one whose blanket impl routes its own context to a
/// provider trait ([`consumer_provider_trait`]). `IsProviderFor`-free, like the rest of the typed
/// resolver's trait recognition.
pub fn is_consumer_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    consumer_provider_trait(tcx, def_id).is_some()
}

/// Recover the `(consumer trait, provider trait)` a component marker keys — the `IsProviderFor`-free
/// inversion the anchors use to turn a `check_components!` / use-site `CanUseComponent<Marker, …>`
/// entry into the real consumer obligation to walk. The provider trait is the one whose provider
/// blanket bounds on `DelegateComponent<marker>` ([`provider_blanket_marker`]); the consumer is the
/// one whose blanket routes to that provider ([`consumer_provider_trait`]). `None` when `marker` is
/// not an ADT, or no provider/consumer pair keys on it.
pub fn marker_to_consumer(tcx: TyCtxt<'_>, marker: Ty<'_>) -> Option<(DefId, DefId)> {
    let provider_did = marker_provider_trait(tcx, marker)?;
    let consumer_did = consumer_of_provider(tcx, provider_did, None)?;
    Some((consumer_did, provider_did))
}

/// The consumer trait whose blanket impl routes to `provider_did` ([`consumer_provider_trait`]),
/// the inverse of that reader. `skip` leaves one trait out of the scan: a detector that runs while
/// the compiler lowers a trait's blanket impl must not read that trait.
pub fn consumer_of_provider(
    tcx: TyCtxt<'_>,
    provider_did: DefId,
    skip: Option<DefId>,
) -> Option<DefId> {
    tcx.all_traits_including_private()
        .filter(|&trait_did| Some(trait_did) != skip)
        .find(|&trait_did| consumer_provider_trait(tcx, trait_did) == Some(provider_did))
}

/// The provider trait a component marker keys — the trait whose provider blanket bounds on
/// `DelegateComponent<marker>` ([`provider_blanket_marker`]). `None` when `marker` is not an ADT,
/// or when no provider trait keys on it, which is what makes this the marker recognizer
/// [`is_component_marker`] as well.
pub fn marker_provider_trait(tcx: TyCtxt<'_>, marker: Ty<'_>) -> Option<DefId> {
    let ty::Adt(marker_def, _) = marker.kind() else {
        return None;
    };
    let marker_did = marker_def.did();
    tcx.all_traits_including_private().find(|&trait_did| {
        matches!(
            provider_blanket_marker(tcx, trait_did).and_then(|marker| marker.ty_adt_def()),
            Some(def) if def.did() == marker_did,
        )
    })
}

/// Whether `ty` is a CGP **component marker** — the `…Component` key some provider trait's
/// delegation blanket bounds on ([`marker_provider_trait`]) — rather than an ordinary type used as
/// a wiring key, such as the `String` of a per-type default or the `u64` of a dispatch table. The
/// distinction is what lets a conflicting key be worded as a component or as a type, and it is
/// structural rather than by name, so a type merely *called* `FooComponent` does not pass.
pub fn is_component_marker(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    marker_provider_trait(tcx, ty).is_some()
}

/// The component marker a CGP **abstract-type** component keys on — `ScalarTypeProviderComponent`
/// for `HasScalarType` — or `None` when `def_id` is not one. This is the name a context writes on the
/// left of the `delegate_components!` entry that chooses the concrete type, so it is what the
/// mismatch leaf's `help` offers as the fix.
///
/// An abstract-type component is recognized **structurally**, by the three things
/// [`#[cgp_type]`](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/reference/macros/cgp_type.md)
/// generates and nothing else does: the trait is a CGP consumer trait
/// ([`consumer_provider_trait`]), its *only* associated item is a type, and its provider trait
/// carries the `UseType<T>` blanket impl that supplies that type. The last condition is the
/// decisive one — it is what makes `UseType<T>` a valid fix to suggest — and it is anchored by
/// `DefId` to the crate defining `UseType`, so a same-named type elsewhere cannot pass. A
/// behavioral component, or a plain trait with an associated type, matches none of the three.
pub fn abstract_type_component_marker<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> Option<Ty<'tcx>> {
    let mut items = tcx.associated_items(def_id).in_definition_order();
    let first = items.next()?;
    if items.next().is_some() || first.kind.tag() != ty::AssocTag::Type {
        return None;
    }
    let provider_did = consumer_provider_trait(tcx, def_id)?;
    // `UseType<T>` is an ADT, so its impl is keyed by self type rather than filed under
    // `blanket_impls()`; scan every impl of the provider trait.
    let supplies_use_type = tcx.all_impls(provider_did).any(|impl_did| {
        match tcx.impl_trait_ref(impl_did).skip_binder().self_ty().kind() {
            ty::Adt(def, _) => is_cgp_item(tcx, def.did(), USE_TYPE_TYPE, CGP_TYPE_CRATE),
            _ => false,
        }
    });
    supplies_use_type.then(|| provider_blanket_marker(tcx, provider_did))?
}

/// The type parameters of the component `key` names, which a redirect appends to its path when it
/// looks the component up. Empty when `key` is not a component marker or the component has none.
/// They are the provider trait's type parameters after its `Self` (the provider) and its context.
pub fn component_params(tcx: TyCtxt<'_>, key: Ty<'_>) -> Vec<String> {
    let Some(provider_did) = marker_provider_trait(tcx, key) else {
        return Vec::new();
    };
    tcx.generics_of(provider_did)
        .own_params
        .iter()
        .filter(|param| matches!(param.kind, ty::GenericParamDefKind::Type { .. }))
        .skip(2)
        .map(|param| param.name.to_string())
        .collect()
}
