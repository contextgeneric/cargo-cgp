//! DefId-anchored recognition of the CGP traits and types the resolver walks.
//!
//! Every stage of the resolution is anchored by `DefId` to the CGP crate that defines the
//! trait or type it matches, so a same-named item from an unrelated crate can never drive a
//! replacement — the same discipline [`component_map`](crate::component_map) uses for
//! `IsProviderFor`.

use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::{CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};

/// Whether `def_id` is a trait/type named `name` defined by crate `krate` — the DefId anchor
/// that keeps a same-named item from an unrelated crate from driving resolution, exactly as
/// `component_map::is_cgp_is_provider_for` does for `IsProviderFor`.
pub(crate) fn is_cgp_item(tcx: TyCtxt<'_>, def_id: DefId, name: &str, krate: &str) -> bool {
    tcx.item_name(def_id).as_str() == name && tcx.crate_name(def_id.krate).as_str() == krate
}

/// The `DefId` of the CGP trait named `name` defined by crate `krate`, or `None` if the crate does
/// not use CGP. Anchored by name *and* crate, like every other CGP lookup here.
pub(crate) fn find_cgp_trait(tcx: TyCtxt<'_>, name: &str, krate: &str) -> Option<DefId> {
    tcx.all_traits_including_private()
        .find(|&did| is_cgp_item(tcx, did, name, krate))
}

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
pub(crate) fn is_provider_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    provider_blanket_marker(tcx, def_id).is_some()
}

/// The component marker a provider trait keys on, read from its provider blanket's `P:
/// DelegateComponent<Marker>` bound — the `IsProviderFor`-free replacement for reading the marker
/// off the `IsProviderFor<Marker, …>` supertrait. `None` when `def_id` has no such blanket (so it
/// is not a provider trait). Also serves as the provider-trait recognizer ([`is_provider_trait`]).
pub(crate) fn provider_blanket_marker<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> Option<Ty<'tcx>> {
    for &impl_did in tcx.trait_impls_of(def_id).blanket_impls() {
        let self_ty = tcx.type_of(impl_did).skip_binder();
        if !matches!(self_ty.kind(), ty::Param(_)) {
            continue;
        }
        for &(clause, _) in tcx.predicates_of(impl_did).predicates {
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
pub(crate) fn consumer_provider_trait(tcx: TyCtxt<'_>, consumer_did: DefId) -> Option<DefId> {
    for &impl_did in tcx.trait_impls_of(consumer_did).blanket_impls() {
        let impl_self = tcx.type_of(impl_did).skip_binder();
        for &(clause, _) in tcx.predicates_of(impl_did).predicates {
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
pub(crate) fn is_consumer_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    consumer_provider_trait(tcx, def_id).is_some()
}

/// Whether `def_id` is a **local blanket-impl trait** — a trait defined in the crate under
/// compilation that carries a blanket impl over a bare context parameter (`impl<Context> Trait for
/// Context where Self: …`). This is the shape `#[cgp_fn]` and `#[blanket_trait]` generate, and the
/// hand-written desugaring of them: the trait is available to any context meeting the blanket's
/// `where` bounds (a `HasField`, another such capability), so a failing `Ctx: Trait` bound has a
/// recoverable root cause down those bounds even though the trait is not a CGP *component* (it has
/// no provider trait or `DelegateComponent`). Requiring the trait to be local excludes std blanket
/// traits such as `Into`, whose blanket lives in another crate, and — together with the auto traits
/// having no such impl — a plain supertrait like `Send`. A CGP consumer trait also matches (its
/// consumer blanket is a blanket impl), which is harmless: callers that care about the distinction
/// test [`consumer_provider_trait`] separately.
pub(crate) fn is_local_blanket_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    def_id.is_local()
        && tcx.is_trait(def_id)
        && !tcx.trait_impls_of(def_id).blanket_impls().is_empty()
}

/// Recover the `(consumer trait, provider trait)` a component marker keys — the `IsProviderFor`-free
/// inversion the anchors use to turn a `check_components!` / use-site `CanUseComponent<Marker, …>`
/// entry into the real consumer obligation to walk. The provider trait is the one whose provider
/// blanket bounds on `DelegateComponent<marker>` ([`provider_blanket_marker`]); the consumer is the
/// one whose blanket routes to that provider ([`consumer_provider_trait`]). `None` when `marker` is
/// not an ADT, or no provider/consumer pair keys on it.
pub(crate) fn marker_to_consumer(tcx: TyCtxt<'_>, marker: Ty<'_>) -> Option<(DefId, DefId)> {
    let ty::Adt(marker_def, _) = marker.kind() else {
        return None;
    };
    let marker_did = marker_def.did();
    let provider_did = tcx.all_traits_including_private().find(|&trait_did| {
        matches!(
            provider_blanket_marker(tcx, trait_did).and_then(|marker| marker.ty_adt_def()),
            Some(def) if def.did() == marker_did,
        )
    })?;
    let consumer_did = tcx
        .all_traits_including_private()
        .find(|&trait_did| consumer_provider_trait(tcx, trait_did) == Some(provider_did))?;
    Some((consumer_did, provider_did))
}

/// Whether `def_id` is a **namespace lookup trait** — the fingerprint every `cgp_namespace!`
/// (and the built-in `DefaultNamespace`/`DefaultImpls*`) generates: a trait whose *only*
/// associated item is a type named `Delegate`, resolving a key to its delegate provider. These are
/// recognized by that structural signature rather than by name or crate, because a user's own
/// namespace trait (`MockNamespace`) lives in the user crate and shares no `DefId` with CGP's, yet
/// must not be mistaken for a regular Rust trait: a failing namespace-lookup bound means the
/// redirect path is unwired, not that some ordinary bound is unmet.
pub(crate) fn is_namespace_lookup_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    if !tcx.is_trait(def_id) {
        return false;
    }
    let mut items = tcx.associated_items(def_id).in_definition_order();
    let Some(first) = items.next() else {
        return false;
    };
    items.next().is_none()
        && first.kind.tag() == ty::AssocTag::Type
        && tcx.item_name(first.def_id).as_str() == "Delegate"
}

/// Decode a CGP `Symbol!` type into its string, by walking the `Chars<'c', Tail>` spine and
/// reading each `char` const argument until `Nil`. Anchored to `cgp_base_types`, and returns
/// `None` for any type that is not a well-formed `Symbol`.
pub(crate) fn decode_symbol(tcx: TyCtxt<'_>, ty: Ty<'_>) -> Option<String> {
    let ty::Adt(def, args) = ty.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), "Symbol", CGP_BASE_TYPES_CRATE) {
        return None;
    }

    // `Symbol<const LEN, Chars>` — the second argument is the head of the `Chars` spine.
    let mut current = args.type_at(1);
    let mut name = String::new();
    loop {
        let ty::Adt(def, args) = current.kind() else {
            return None;
        };
        if is_cgp_item(tcx, def.did(), "Nil", CGP_BASE_TYPES_CRATE) {
            break;
        }
        if !is_cgp_item(tcx, def.did(), "Chars", CGP_BASE_TYPES_CRATE) {
            return None;
        }

        // `Chars<const CHAR: char, Tail>` — read the char, then follow the tail.
        let scalar = args.const_at(0).try_to_value()?.valtree.try_to_leaf()?;
        name.push(char::from_u32(scalar.to_u32())?);
        current = args.type_at(1);
    }
    Some(name)
}
