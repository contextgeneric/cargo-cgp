//! The shapes that tell an unmet `DelegateComponent` on a delegation table from a routing dead-end.

use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::resolve::query::owner_has_impl_of;

/// Whether an unmet `DelegateComponent` on `owner` was reached through the delegation blanket of
/// `parent`, an obligation on `owner` itself, although `owner` has a concrete impl of `parent`'s
/// trait: the blanket was tried only because that impl did not unify.
pub(crate) fn is_blanket_dead_end<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: Ty<'tcx>,
    parent: ty::TraitRef<'tcx>,
) -> bool {
    tcx.erase_and_anonymize_regions(parent.self_ty()) == owner
        && owner_has_impl_of(tcx, parent.def_id, owner)
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
