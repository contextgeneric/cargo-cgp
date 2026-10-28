//! Whether a type is the concrete `Self` of some impl of a trait.

use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;

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
pub fn owner_has_impl_of<'tcx>(tcx: TyCtxt<'tcx>, trait_did: DefId, owner: Ty<'tcx>) -> bool {
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
