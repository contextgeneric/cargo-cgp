//! What serves a slot a misused CGP core provider cannot.

use cargo_cgp_error_processing::CoreProviderHint;
use rustc_middle::ty::{self, Ty, TyCtxt, Upcast as _};

use crate::config::{
    CGP_COMPONENT_CRATE, CGP_FIELD_CRATE, REDIRECT_LOOKUP_TYPE, USE_DELEGATE_TYPE, USE_FIELD_TYPE,
    USE_FIELDS_TYPE,
};
use crate::resolve::classify::known_obligation;
use crate::resolve::label::render_ty;
use crate::resolve::query::{holds, is_cgp_item, provider_blanket_marker};

/// What serves the slot of `parent`, a provider-trait obligation `Provider: Trait<Context, …>`,
/// where the wired `Provider` is a CGP core provider used where it has no impl. The core providers
/// are anchored by `DefId` to the crates that define them, and each fix is confirmed against the
/// program:
///
/// - `UseField<Tag>` on a trait with no `UseField` impl, when `UseFields` implements the trait for
///   the context: a multi-method getter, which reads each method's field by the method's name.
/// - `UseDelegate<Table>` on a component with parameters, when the trait has a `RedirectLookup`
///   impl: an `open` statement dispatches on the parameters without `#[derive_delegate]`.
///
/// `None` for any other provider, and when `parent` is not fully known.
pub fn core_provider_hint<'tcx>(
    tcx: TyCtxt<'tcx>,
    parent: ty::TraitRef<'tcx>,
) -> Option<CoreProviderHint> {
    let parent = known_obligation(tcx, parent)?;
    let ty::Adt(provider_def, _) = parent.self_ty().kind() else {
        return None;
    };
    let provider_is = |name: &str, krate: &str| is_cgp_item(tcx, provider_def.did(), name, krate);
    let impl_self_is = |name: &str| {
        tcx.all_impls(parent.def_id).find_map(|impl_did| {
            match tcx.impl_trait_ref(impl_did).skip_binder().self_ty().kind() {
                ty::Adt(def, _) if is_cgp_item(tcx, def.did(), name, CGP_COMPONENT_CRATE) => {
                    Some(*def)
                }
                _ => None,
            }
        })
    };
    if provider_is(USE_FIELD_TYPE, CGP_FIELD_CRATE) {
        let use_fields = impl_self_is(USE_FIELDS_TYPE)?;
        let candidate = Ty::new_adt(tcx, use_fields, ty::GenericArgs::empty());
        let mut args = parent.args.to_vec();
        args[0] = candidate.into();
        let applied = ty::TraitRef::new_from_args(tcx, parent.def_id, tcx.mk_args(&args));
        return holds(tcx, ty::Binder::dummy(applied).upcast(tcx))
            .then_some(CoreProviderHint::UseFields);
    }
    if provider_is(USE_DELEGATE_TYPE, CGP_COMPONENT_CRATE) {
        impl_self_is(REDIRECT_LOOKUP_TYPE)?;
        let generics = tcx.generics_of(parent.def_id);
        let params: Vec<String> = generics
            .own_params
            .iter()
            .skip(2)
            .filter(|param| matches!(param.kind, ty::GenericParamDefKind::Type { .. }))
            .map(|param| param.name.to_string())
            .collect();
        if params.is_empty() {
            return None;
        }
        let marker = provider_blanket_marker(tcx, parent.def_id)?;
        let component = render_ty(tcx, marker);
        return Some(CoreProviderHint::OpenDispatch { component, params });
    }
    None
}
