//! Telling a context from a provider struct and from an ordinary type.

use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::query::{find_cgp_trait, is_provider_trait};

/// Whether `ty` is a struct or enum defined in the crate being compiled. A context may also come
/// from another crate ([`is_context_adt`]).
pub fn is_local_adt(ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if def.did().is_local())
}

/// Whether `ty` is a struct or enum the resolver can re-check as a context: one the crate being
/// compiled defines, or one from another crate that carries CGP wiring of its own (a
/// `DelegateComponent` impl keyed on it), as a context a library defines and exports does. The
/// wiring evidence keeps a foreign type that is no context, such as `Vec`, from being read as one.
pub fn is_context_adt(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    let ty::Adt(def, _) = ty.kind() else {
        return false;
    };
    def.did().is_local() || carries_wiring(tcx, ty)
}

/// Whether `ty` is a type with `DelegateComponent` wiring of its own, an impl whose `Self` is this
/// type: what makes a type a *context*, the word a message uses only for such a type. A type
/// argument a consumer trait is required of, with no wiring, is named plainly instead.
pub fn carries_wiring(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    let ty::Adt(def, _) = ty.kind() else {
        return false;
    };
    let Some(delegate_component) =
        find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
    else {
        return false;
    };
    tcx.all_impls(delegate_component).any(|impl_did| {
        matches!(
            tcx.impl_trait_ref(impl_did).skip_binder().self_ty().kind(),
            ty::Adt(self_def, _) if self_def.did() == def.did(),
        )
    })
}

/// Whether `ty` is a *provider* struct — the concrete `Self` of some provider-trait impl — rather
/// than a context. A provider is never a context, so a use-site recovery must not treat one as the
/// failing context even when it merely shares a diagnostic span (rustc names the provider in a
/// "required for … to implement …" note, whose span can fall inside the provider's own definition).
/// The delegation blanket `impl<Ctx, P> ProviderTrait<Ctx> for P` has a *type parameter* `Self`, not
/// an ADT, so it never matches here.
pub fn is_provider_struct(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
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
