//! Whether a type is the concrete `Self` of some impl of a trait.

use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;

/// Whether `owner`'s ADT appears as the concrete `Self` of any impl of `trait_did`: whether the
/// type implements the trait itself, as opposed to reaching it only through a blanket. The resolver
/// asks it of `DelegateComponent`, where an impl makes the owner a context or a delegation table,
/// of a provider trait, where a provider with its own impl that the walk reached through the
/// delegation blanket is a dead-end and one without is not a provider, and of the extensible-data
/// traits.
///
/// A blanket impl (whose `Self` is a bare type parameter, like the CGP delegation blanket) is not a
/// concrete `Self`, so it never counts; only an `impl … for SomeAdt` does.
pub fn owner_has_impl_of(tcx: TyCtxt<'_>, trait_did: DefId, owner: Ty<'_>) -> bool {
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
