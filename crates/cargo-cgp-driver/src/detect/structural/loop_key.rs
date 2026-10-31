//! A `for` loop in a wiring table whose key does not use the loop's key variable.

use cargo_cgp_error_processing::{StructuralMistake, WiringKey};
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT};
use crate::resolve::query::{
    describe_key, is_cgp_item, is_namespace_lookup_trait, local_trait_impls, same_range,
};

/// The `for` loop an `E0207` at `primary_span` is about, when the unconstrained parameter there
/// belongs to an entry a loop generated. `for <T, Provider> in Table { Key: Provider }` lowers to a
/// `DelegateComponent<Key>` impl on the context (and its `IsProviderFor` twin) bounded by
/// `T: Table<…, Delegate = Provider>`, so a key that does not mention `T` leaves both variables
/// unconstrained, and rustc reports each. The loop's key variable is the `Self` of that namespace
/// bound. `None` when the impl declaring the parameter is not such an entry.
pub fn detect_loop_key_without_variable(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<StructuralMistake> {
    for local in local_trait_impls(tcx) {
        let impl_did = local.to_def_id();
        let declares_param = tcx
            .generics_of(impl_did)
            .own_params
            .iter()
            .any(|param| same_range(tcx.def_span(param.def_id), primary_span));
        if !declares_param {
            continue;
        }
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        let is_entry = [DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT]
            .iter()
            .any(|name| is_cgp_item(tcx, trait_ref.def_id, name, CGP_COMPONENT_CRATE));
        if !is_entry {
            continue;
        }
        // `DelegateComponent<Key> for Context` and `IsProviderFor<Key, …> for Context` both carry
        // the key as their first argument after `Self`.
        let key = trait_ref.args.type_at(1);
        let variable = tcx
            .clauses_of(impl_did)
            .clauses
            .iter()
            .filter_map(|(clause, _)| clause.as_trait_clause())
            .map(|bound| bound.skip_binder())
            .find(|bound| {
                matches!(bound.self_ty().kind(), ty::Param(_))
                    && is_namespace_lookup_trait(tcx, bound.def_id())
                    && !key.walk().any(|arg| arg.as_type() == Some(bound.self_ty()))
            })?
            .self_ty();
        // An `E0207` is emitted before anything has printed a type with trimmed paths, and the
        // first such print records itself on the diagnostic context, whose lock the emitter holds.
        // So these print each type by its own name, which needs no such record; two same-named
        // types in different modules then read alike.
        return with_forced_trimmed_paths!({
            let key = match describe_key(tcx, key, impl_did)? {
                WiringKey::Path(text) | WiringKey::Component(text) | WiringKey::Type(text) => text,
                WiringKey::Blanket(_) => return None,
            };
            Some(StructuralMistake::LoopKeyWithoutVariable {
                context: trait_ref.self_ty().to_string(),
                variable: variable.to_string(),
                key,
            })
        });
    }
    None
}
