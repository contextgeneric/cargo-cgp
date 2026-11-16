//! Building the component-marker → trait-names map from the compiler, for the rustc-free
//! [`rewrite`](cargo_cgp_error_processing::rewrite) module.
//!
//! See `cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md` for the two
//! links it inverts, the `DefId` anchor, and why entries are keyed by full path.

use std::collections::HashMap;

use cargo_cgp_error_processing::rewrite::ComponentTraitNames;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::{CGP_COMPONENT_CRATE, IS_PROVIDER_FOR_TRAIT};

/// Build the component-name map from the `TyCtxt` currently in thread-local scope, returning
/// an empty map if none is present.
///
/// This is the `fn`-pointer initializer the driver hands to
/// [`ComponentNameMap`](cargo_cgp_error_processing::rewrite::ComponentNameMap): the emitter
/// installs it at session setup, before any `TyCtxt` exists, and it is forced only on the first
/// message rewrite, which happens during trait solving, when a `TyCtxt` *is* in scope
/// ([`rustc_middle::ty::tls`]). Being a plain `fn` (capturing nothing, reading TLS) lets the
/// rustc-free `ComponentNameMap` hold it.
pub fn build_name_map_from_tls() -> HashMap<String, ComponentTraitNames> {
    ty::tls::with_opt(|tcx| tcx.map(build_component_name_map).unwrap_or_default())
}

/// Whether `def_id` is the `IsProviderFor` trait *defined by* the `cgp-component` crate, not
/// merely a trait spelled `IsProviderFor`. The name is checked first because it is the cheaper
/// query and rules out almost every bound before the crate is even consulted.
fn is_cgp_is_provider_for(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    tcx.item_name(def_id).as_str() == IS_PROVIDER_FOR_TRAIT
        && tcx.crate_name(def_id.krate).as_str() == CGP_COMPONENT_CRATE
}

/// Build the map from component-marker name to the consumer/provider trait names behind it,
/// by inverting the `IsProviderFor` supertrait and consumer-blanket-impl links across every
/// trait in scope. Returns an empty map when no trait carries a genuine `cgp-component`
/// `IsProviderFor` supertrait: the case for any compilation that does not use CGP.
pub fn build_component_name_map(tcx: TyCtxt<'_>) -> HashMap<String, ComponentTraitNames> {
    let all_traits: Vec<DefId> = tcx.all_traits_including_private().collect();

    let providers = provider_names(tcx, &all_traits);
    let consumers = consumer_names(tcx, &all_traits, &providers);

    providers
        .into_iter()
        .map(|(provider_did, (component, provider))| {
            // Fall back to the provider trait name if no consumer blanket impl is found, so
            // a partially-recognized component still improves the provider-side note.
            let consumer = consumers
                .get(&provider_did)
                .cloned()
                .unwrap_or_else(|| provider.clone());
            // A provider trait's type parameters are `Self`, the context, then the component's.
            let params = tcx
                .generics_of(provider_did)
                .own_params
                .iter()
                .filter(|param| matches!(param.kind, ty::GenericParamDefKind::Type { .. }))
                .count()
                .saturating_sub(2);
            (
                component,
                ComponentTraitNames {
                    consumer,
                    provider,
                    params,
                },
            )
        })
        .collect()
}

/// Provider trait DefId → (component marker name, provider trait name), found by the
/// genuine `IsProviderFor<Marker, …>` supertrait every provider trait carries. Checking
/// each supertrait's identity directly (rather than first picking "the" `IsProviderFor` by
/// name) means a same-named trait from another crate is simply never matched.
fn provider_names(tcx: TyCtxt<'_>, all_traits: &[DefId]) -> HashMap<DefId, (String, String)> {
    let mut providers: HashMap<DefId, (String, String)> = HashMap::new();
    for &trait_did in all_traits {
        for &(clause, _) in tcx.explicit_super_clauses_of(trait_did).skip_binder() {
            let Some(predicate) = clause.as_trait_clause() else {
                continue;
            };
            let trait_ref = predicate.skip_binder().trait_ref;
            if !is_cgp_is_provider_for(tcx, trait_ref.def_id) {
                continue;
            }
            // Args are `[Self, Marker, Context, Params]`; the marker is the second.
            let Some(marker_ty) = trait_ref.args.get(1).and_then(|arg| arg.as_type()) else {
                continue;
            };
            if let ty::Adt(marker, _) = marker_ty.kind() {
                providers.insert(
                    trait_did,
                    (
                        // Full path, not the bare name, so same-named markers in different
                        // modules key distinct entries.
                        tcx.def_path_str(marker.did()),
                        tcx.item_name(trait_did).to_string(),
                    ),
                );
            }
        }
    }
    providers
}

/// Provider trait DefId → consumer trait name, found by the consumer's blanket impl
/// `impl<C> Consumer for C where C: Provider<C>`: a blanket impl bounding its *own* self type on a
/// known provider trait belongs to that provider's consumer. Requiring the bound's self type to be
/// the impl's self type tells this apart from the provider blanket impl, which also bounds the same
/// provider trait but on a projected `<C as DelegateComponent<…>>::Delegate` rather than on `C`
/// itself.
fn consumer_names(
    tcx: TyCtxt<'_>,
    all_traits: &[DefId],
    providers: &HashMap<DefId, (String, String)>,
) -> HashMap<DefId, String> {
    let mut consumers: HashMap<DefId, String> = HashMap::new();
    for &consumer_did in all_traits {
        for &impl_did in tcx.trait_impls_of(consumer_did).blanket_impls() {
            let impl_self_ty = tcx.type_of(impl_did).skip_binder();
            for &(clause, _) in tcx.clauses_of(impl_did).clauses {
                let Some(predicate) = clause.as_trait_clause() else {
                    continue;
                };
                let trait_ref = predicate.skip_binder().trait_ref;
                if providers.contains_key(&trait_ref.def_id) && trait_ref.self_ty() == impl_self_ty
                {
                    consumers.insert(trait_ref.def_id, tcx.item_name(consumer_did).to_string());
                }
            }
        }
    }
    consumers
}
