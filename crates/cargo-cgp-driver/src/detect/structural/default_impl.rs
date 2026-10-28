//! A `#[default_impl]` on a provider impl that is generic over a parameter.

use cargo_cgp_error_processing::{StructuralMistake, WiringKey};
use rustc_hir::def::DefKind;
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{self, TyCtxt, TypeVisitableExt as _};
use rustc_span::Span;

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::detect::conflict::{impl_delegate_type, same_range};
use crate::resolve::cgp_item::{is_cgp_item, is_namespace_lookup_trait};
use crate::resolve::query::describe_key;

/// The `#[default_impl]` an `E0207` at `primary_span` is about. The registration it generates,
/// `impl<T, __Components__> Table<…, __Components__> for Key { type Delegate = Provider; }`,
/// copies the provider impl's generic parameters, so a parameter the key and the table do not
/// mention is unconstrained; the caret is on that parameter in the provider's impl. Recognized as
/// a local impl of a namespace lookup trait (not `DelegateComponent`) declaring the parameter at the
/// caret, which neither its key nor its trait arguments use. A provider struct that uses the
/// parameter (`ShowWith<T>`) is recognized only for a type key, whose type then stands in for the
/// parameter in the fix (`ShowWith<String>`). `None` otherwise.
pub fn detect_generic_default_impl(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<StructuralMistake> {
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let impl_did = local.to_def_id();
        let Some(param) = tcx
            .generics_of(impl_did)
            .own_params
            .iter()
            .find(|param| same_range(tcx.def_span(param.def_id), primary_span))
        else {
            continue;
        };
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        // `DelegateComponent` has a namespace trait's shape too, but a per-entry generic in a
        // wiring table is a different mistake.
        if is_cgp_item(
            tcx,
            trait_ref.def_id,
            DELEGATE_COMPONENT_TRAIT,
            CGP_COMPONENT_CRATE,
        ) || !is_namespace_lookup_trait(tcx, trait_ref.def_id)
        {
            continue;
        }
        let uses_param = |arg: ty::GenericArg<'_>| {
            arg.walk()
                .any(|inner| match inner.as_type().map(|ty| *ty.kind()) {
                    Some(ty::Param(used)) => used.index == param.index,
                    _ => false,
                })
        };
        if trait_ref.args.iter().any(uses_param) {
            continue;
        }
        // The registration names the provider struct. When the struct itself uses the parameter,
        // the fix registers it at the key, with the key's type in the parameter's place.
        let provider = impl_delegate_type(tcx, impl_did)?;
        if !matches!(provider.kind(), ty::Adt(..)) {
            continue;
        }
        let provider_at_key = if uses_param(provider.into()) {
            let key_ty = trait_ref.self_ty();
            let args = ty::GenericArgs::for_item(tcx, impl_did, |def, _| {
                if def.index == param.index {
                    key_ty.into()
                } else {
                    tcx.mk_param_from_def(def)
                }
            });
            let at_key = ty::EarlyBinder::bind(tcx, provider)
                .instantiate(tcx, args)
                .skip_norm_wip();
            if at_key.has_param() {
                continue;
            }
            Some(at_key)
        } else {
            None
        };
        // Printed by each type's own name, as an `E0207` arrives before a trimmed print is safe
        // (see the loop detector).
        return with_forced_trimmed_paths!({
            let key = match describe_key(tcx, trait_ref.self_ty(), impl_did)? {
                WiringKey::Type(text) => text,
                WiringKey::Path(text) | WiringKey::Component(text) if provider_at_key.is_none() => {
                    text
                }
                _ => return None,
            };
            Some(StructuralMistake::GenericDefaultImpl {
                provider: provider.to_string(),
                key,
                parameter: param.name.to_string(),
                provider_at_key: provider_at_key.map(|at_key| at_key.to_string()),
            })
        });
    }
    None
}
