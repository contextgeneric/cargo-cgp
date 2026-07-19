//! Whether a terminal leaf is a real root cause or a routing dead-end to drop.

use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT,
};
use crate::resolve::cgp_item::is_cgp_item;

/// Whether a terminal leaf is a real root cause worth reporting, rather than pure wiring plumbing.
/// A `CanUseComponent` or `IsProviderFor` that bottoms out unmet is a routing dead-end (the real
/// cause sits down another branch), so it is dropped instead of shown. An unmet `DelegateComponent`
/// is a real root cause in three shapes:
///
/// - on the **context** itself, the context does not wire the component (a
///   [`Leaf::MissingWiring`](cargo_cgp_error_processing::Leaf::MissingWiring));
/// - as a **dispatch lookup into a separate table** — the recognized-structurally case: the
///   obligation is `Components: DelegateComponent<Key>` where `Components` is a *proper part* of the
///   parent obligation's `Self` (as `Components` is of `UseDelegate<Components>` /
///   `UseInputDelegate<Components>`, or any provider that dispatches through a table it holds as a
///   parameter). Such a `where`-clause is unambiguously a table lookup — its owner is not the
///   provider itself — so an unmet one is a missing entry regardless of whether that table wires any
///   other key ([`is_dispatch_lookup`]); this is what reaches an *empty* dispatch table, which the
///   owner-property check cannot see;
/// - on a **delegation table reached via the generic blanket** — an aggregate provider whose own
///   table lacks a key, recognized because the owner wires *some* other key
///   ([`owner_has_impl_of`] for `DelegateComponent`).
///
/// It is dropped only when it is none of these. The case that makes the last gate load-bearing (not
/// just cautious): a **leaf provider** whose concrete impl fixes an input type the walk cannot match.
/// A pipeline stage like `HandleShout` (`impl Handler<Code, String>`) fed an *unknown* input — a
/// call-site placeholder an earlier stage's `::Output` never resolved — does not unify with its
/// concrete impl, so `impl_where_obligations` falls through to the delegation blanket and produces an
/// unmet `HandleShout: DelegateComponent<HandlerComponent>`. There the owner *is* the parent's `Self`
/// (the blanket keys on the provider itself, not a separate table), so `is_dispatch_lookup` is
/// false, and `HandleShout` wires nothing, so the owner-property check is false too — a dead-end,
/// correctly dropped, since `HandleShout` is a valid provider and simply is not a table.
///
/// A `DelegateComponent` that is *none* of the above splits one more way, by whether the owner is
/// genuinely a provider for the parent trait at all. It arises via the generic blanket for the
/// parent provider trait `T` (owner == parent `Self`), reached because no concrete `impl T for
/// owner` unified. Two sub-cases, told apart by whether such a concrete impl *exists*:
///
/// - the owner **has** a concrete impl of `T` that merely did not unify (a leaf provider fed the
///   wrong input, the `HandleShout` dead-end) — dropped, its real cause runs through that impl;
/// - the owner has **no** concrete impl of `T` at all — it is genuinely not a provider, wired where
///   one was expected (`UseBasicAuth<QueryBalanceRequest>`, a request type in a handler slot) —
///   reported as a [`Leaf::NotAProvider`](cargo_cgp_error_processing::Leaf::NotAProvider).
///
/// `parent` is the obligation one hop above the leaf (the impl whose `where`-clause produced it), or
/// `None` at the root: its `Self` tells the separate-table lookup from the self-keyed blanket, and
/// its trait is the `T` the not-a-provider check and leaf name against.
pub(crate) fn is_reportable_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    context: Ty<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
) -> bool {
    let did = leaf_ref.def_id;
    if is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE) {
        let self_ty = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
        if self_ty == context
            || parent.is_some_and(|p| is_dispatch_lookup(tcx, self_ty, p.self_ty()))
            || owner_has_impl_of(tcx, did, self_ty)
        {
            return true;
        }
        // Reached via the generic blanket for the parent provider trait: report only when the owner
        // has no concrete impl of that trait (a genuine non-provider), never when it has one that
        // merely failed to unify (a valid provider reached by an input mismatch — a dead-end).
        return parent.is_some_and(|p| !owner_has_impl_of(tcx, p.def_id, self_ty));
    }
    !is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        && !is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
}

/// Whether an unmet `DelegateComponent` on `owner` is a **dispatch lookup into a separate table**:
/// `owner` is a *proper subterm* of `parent_self`, the `Self` of the provider impl whose
/// `where`-clause introduced the lookup. This is the shape every dispatcher provider shares —
/// `UseDelegate<Components>` / `UseInputDelegate<Components>` and any custom dispatcher hold their
/// table as a parameter and look a key up in it (`Components: DelegateComponent<Key>`), so the lookup's
/// owner is *inside* the provider's `Self`, never equal to it. The generic delegation blanket, by
/// contrast, keys on the provider *itself* (`P: DelegateComponent<Marker>` with `Self = P`), where
/// owner equals `parent_self` — so this returns `false` for it, leaving the blanket cases to the
/// context and owner-property checks. Recognizing the lookup structurally means an entry-less
/// table is still reported, where the owner-property heuristic (which needs a wired key to find) would
/// miss it.
pub(crate) fn is_dispatch_lookup<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: Ty<'tcx>,
    parent_self: Ty<'tcx>,
) -> bool {
    let parent_self = tcx.erase_and_anonymize_regions(parent_self);
    parent_self != owner
        && parent_self
            .walk()
            .filter_map(|arg| arg.as_type())
            .any(|sub| tcx.erase_and_anonymize_regions(sub) == owner)
}

/// Whether `owner`'s ADT appears as the concrete `Self` of any impl of `trait_did`. This backs the
/// two "is this owner meant to X" checks the `DelegateComponent` classification needs, each keyed on
/// a different trait:
///
/// - with the **`DelegateComponent`** trait: whether `owner` is a *delegation table* — an aggregate
///   provider or a `UseDelegate`/`UseInputDelegate` inner table wiring at least one key. Any one
///   entry is enough; the owner it excludes is one with zero delegation impls.
/// - with the **parent provider trait**: whether `owner` has a *concrete impl of that provider
///   trait*, which tells a leaf provider reached via the blanket by an input mismatch (has one — a
///   dead-end) from a genuine non-provider (has none — a
///   [`Leaf::NotAProvider`](cargo_cgp_error_processing::Leaf::NotAProvider)).
///
/// A blanket impl (whose `Self` is a bare type parameter, like the CGP delegation blanket) is not a
/// concrete `Self`, so it never counts — only an `impl … for SomeAdt` does.
pub(crate) fn owner_has_impl_of<'tcx>(
    tcx: TyCtxt<'tcx>,
    trait_did: DefId,
    owner: Ty<'tcx>,
) -> bool {
    let ty::Adt(owner_def, _) = owner.kind() else {
        return false;
    };
    tcx.all_impls(trait_did).any(|impl_did| {
        matches!(
            tcx.impl_trait_ref(impl_did).skip_binder().self_ty().kind(),
            ty::Adt(def, _) if def.did() == owner_def.did()
        )
    })
}
