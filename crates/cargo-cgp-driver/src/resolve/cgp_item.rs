//! DefId-anchored recognition of the CGP traits and types the resolver walks.
//!
//! Every stage of the resolution is anchored by `DefId` to the CGP crate that defines the
//! trait or type it matches, so a same-named item from an unrelated crate can never drive a
//! replacement — the same discipline [`component_map`](crate::component_map) uses for
//! `IsProviderFor`.

use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;

use crate::config::{
    CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, CGP_CRATES, CGP_TYPE_CRATE,
    DELEGATE_COMPONENT_TRAIT, NIL_TYPE, PATH_CONS_TYPE, USE_TYPE_TYPE,
};

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

/// Whether `def_id` is a **capability trait** — one carrying a blanket impl over a bare context
/// parameter (`impl<Context> Trait for Context where Self: …`). This is the shape `#[cgp_fn]` and
/// `#[blanket_trait]` generate, and the hand-written desugaring of them: the trait is available to
/// any context meeting the blanket's `where` bounds (a `HasField`, another such capability), so a
/// failing `Ctx: Trait` bound has a recoverable root cause down those bounds even though the trait
/// is not a CGP *component* (it has no provider trait or `DelegateComponent`). A CGP consumer trait
/// also matches (its consumer blanket is a blanket impl), which is harmless: callers that care about
/// the distinction test [`consumer_provider_trait`] separately.
///
/// The blanket impl alone is far too broad a signal to accept, since `ToString`, `Into`, and
/// `Borrow` all carry one and reshaping their failures into CGP errors would be an over-reach. So a
/// trait qualifies one of two ways. A trait the **checked crate defines** qualifies outright:
/// cargo-cgp runs on CGP workspaces, and a local blanket trait whose bound is failing is the shape
/// `#[cgp_fn]` produces. A **foreign** trait must instead show that its blanket genuinely depends on
/// CGP ([`blanket_depends_on_cgp`]) — which is what admits a capability a library publishes, the
/// normal arrangement, while still excluding the std blankets the locality rule was aimed at.
pub(crate) fn is_capability_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    has_blanket_impl(tcx, def_id) && (def_id.is_local() || blanket_depends_on_cgp(tcx, def_id, 0))
}

/// Whether `def_id` is a trait carrying at least one blanket impl.
fn has_blanket_impl(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    tcx.is_trait(def_id) && !tcx.trait_impls_of(def_id).blanket_impls().is_empty()
}

/// How far the CGP-evidence search follows one capability's blanket bounds into another's. A
/// composed chain (`Describe` → `HasName` → `HasField`) is a few links at most, and the bound is
/// also what stops a pair of capabilities that depend on each other from looping.
const MAX_CAPABILITY_DEPTH: u32 = 4;

/// Whether a blanket-impl trait's blanket actually depends on CGP — the positive evidence a
/// *foreign* trait needs before it is read as a capability. A bound qualifies when its trait comes
/// from one of [`CGP_CRATES`] (`HasField` above all), is a CGP consumer trait (recognized
/// structurally in any crate), or is itself such a capability, followed to
/// [`MAX_CAPABILITY_DEPTH`].
fn blanket_depends_on_cgp(tcx: TyCtxt<'_>, def_id: DefId, depth: u32) -> bool {
    if depth >= MAX_CAPABILITY_DEPTH {
        return false;
    }
    tcx.trait_impls_of(def_id)
        .blanket_impls()
        .iter()
        .any(|&blanket| {
            tcx.predicates_of(blanket)
                .predicates
                .iter()
                .filter_map(|(clause, _)| clause.as_trait_clause())
                .any(|bound| {
                    let bound_did = bound.def_id();
                    is_cgp_crate_trait(tcx, bound_did)
                        || is_consumer_trait(tcx, bound_did)
                        || (has_blanket_impl(tcx, bound_did)
                            && blanket_depends_on_cgp(tcx, bound_did, depth + 1))
                })
        })
}

/// Whether `def_id` is defined by one of CGP's own crates ([`CGP_CRATES`]) — anchored to the
/// defining crate, so a same-named trait elsewhere never counts as evidence.
fn is_cgp_crate_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    let krate = tcx.crate_name(def_id.krate);
    CGP_CRATES.iter().any(|name| krate.as_str() == *name)
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

/// The component marker a CGP **abstract-type** component keys on — `ScalarTypeProviderComponent`
/// for `HasScalarType` — or `None` when `def_id` is not one. This is the name a context writes on the
/// left of the `delegate_components!` entry that chooses the concrete type, so it is what the
/// mismatch leaf's `help` offers as the fix.
///
/// An abstract-type component is recognized **structurally**, by the three things
/// [`#[cgp_type]`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/cgp_type.md)
/// generates and nothing else does: the trait is a CGP consumer trait ([`consumer_provider_trait`]),
/// its *only* associated item is a type, and its provider trait carries the `UseType<T>` blanket impl
/// that supplies that type. The last condition is the decisive one — it is what makes `UseType<T>` a
/// valid fix to suggest — and it is anchored by `DefId` to the crate defining `UseType`, so a
/// same-named type elsewhere cannot pass. A behavioral component, or a plain trait with an associated
/// type, matches none of the three.
pub(crate) fn abstract_type_component_marker<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: DefId,
) -> Option<Ty<'tcx>> {
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

/// Whether `ty` is a struct or enum defined in the crate being compiled — the only kind of type
/// whose wiring the resolver re-checks as a context.
pub(crate) fn is_local_adt(ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if def.did().is_local())
}

/// Whether `ty` is a *provider* struct — the concrete `Self` of some provider-trait impl — rather
/// than a context. A provider is never a context, so a use-site recovery must not treat one as the
/// failing context even when it merely shares a diagnostic span (rustc names the provider in a
/// "required for … to implement …" note, whose span can fall inside the provider's own definition).
/// The delegation blanket `impl<Ctx, P> ProviderTrait<Ctx> for P` has a *type parameter* `Self`, not
/// an ADT, so it never matches here.
pub(crate) fn is_provider_struct(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    let ty::Adt(def, _) = ty.kind() else {
        return false;
    };
    let adt_did = def.did();
    tcx.all_local_trait_impls(())
        .iter()
        .filter(|(trait_did, _)| is_provider_trait(tcx, **trait_did))
        .flat_map(|(_, impls)| impls.iter())
        .any(|&impl_did| {
            matches!(
                tcx.impl_trait_ref(impl_did.to_def_id()).skip_binder().self_ty().kind(),
                ty::Adt(self_def, _) if self_def.did() == adt_did,
            )
        })
}

/// Whether `ty` is CGP's type-level path spine `PathCons<…>` — an `open`/namespace redirect key,
/// as opposed to a bare component marker. Anchored by `DefId` to [`CGP_BASE_TYPES_CRATE`].
pub(crate) fn is_path_cons(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if is_cgp_item(tcx, def.did(), PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE))
}

/// The `(head, tail)` of a `PathCons<Head, Tail>` type, or `None` when `ty` is not a `PathCons`.
pub(crate) fn path_cons_parts<'tcx>(
    tcx: TyCtxt<'tcx>,
    ty: Ty<'tcx>,
) -> Option<(Ty<'tcx>, Ty<'tcx>)> {
    match ty.kind() {
        ty::Adt(def, args) if is_cgp_item(tcx, def.did(), PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE) => {
            Some((args.type_at(0), args.type_at(1)))
        }
        _ => None,
    }
}

/// A redirect path with its trailing **unknown** segments removed, or `None` when it has none.
///
/// A `RedirectLookup` keys its table on the redirect path *plus the component's own parameters*, so
/// a `Handler` lookup for the fragment `Missing` reads `@…HandlerComponent.Missing.<Input>`. When
/// the failure was recovered from a call the code does not fully type — a
/// [call site](crate::resolve::call_site) whose input is inferred — that trailing parameter is a
/// placeholder, and reporting it would name an entry the programmer could not write, besides being
/// dropped as unknowable. Trimming the trailing run leaves the path they *can* wire.
///
/// Only a trailing run is trimmed: a placeholder further up is part of what the lookup keys on, so
/// such a leaf stays unknowable and is dropped as before.
pub(crate) fn trim_unknown_path_tail<'tcx>(tcx: TyCtxt<'tcx>, path: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let ty::Adt(cons_def, _) = path.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, cons_def.did(), PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE) {
        return None;
    }

    let mut segments = Vec::new();
    let mut rest = path;
    while let Some((head, tail)) = path_cons_parts(tcx, rest) {
        segments.push(head);
        rest = tail;
    }
    if !is_nil(tcx, rest) {
        return None;
    }

    let kept = segments
        .iter()
        .rposition(|segment| !segment.has_placeholders())
        .map_or(0, |last| last + 1);
    if kept == segments.len() {
        return None;
    }

    Some(segments[..kept].iter().rev().fold(rest, |tail, &head| {
        Ty::new_adt(tcx, *cons_def, tcx.mk_args(&[head.into(), tail.into()]))
    }))
}

/// Whether `ty` is CGP's type-level path/list terminator `Nil`.
pub(crate) fn is_nil(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if is_cgp_item(tcx, def.did(), NIL_TYPE, CGP_BASE_TYPES_CRATE))
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
