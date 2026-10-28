//! Looking up components by name across the crate graph.
//!
//! A component is found through its provider trait: `#[cgp_component]` gives every provider trait a
//! delegation blanket bounded on `DelegateComponent<Marker>`, and [`provider_blanket_marker`] reads
//! the marker off it. Scanning every trait the crate can see therefore yields every component,
//! local or from a dependency, with its provider trait.

use rustc_hir::def::DefKind;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::DefId;

use crate::resolve::cgp_item::{
    consumer_provider_trait, is_consumer_trait, provider_blanket_marker,
};
use crate::resolve::query::public_dependency_path;

/// Every component marker the crate can see, each with its provider trait.
pub fn component_markers(tcx: TyCtxt<'_>) -> Vec<(DefId, DefId)> {
    tcx.all_traits_including_private()
        .filter_map(|trait_did| {
            let marker = provider_blanket_marker(tcx, trait_did)?;
            let ty::Adt(marker_def, _) = marker.kind() else {
                return None;
            };
            Some((marker_def.did(), trait_did))
        })
        .collect()
}

/// The component of the consumer trait `consumer_did`: the marker its provider trait's delegation
/// blanket keys on.
pub fn consumer_component(tcx: TyCtxt<'_>, consumer_did: DefId) -> Option<DefId> {
    let provider_did = consumer_provider_trait(tcx, consumer_did)?;
    let marker = provider_blanket_marker(tcx, provider_did)?;
    match marker.kind() {
        ty::Adt(marker_def, _) => Some(marker_def.did()),
        _ => None,
    }
}

/// The one consumer trait named `name`, or `None` when there is none or more than one.
pub fn consumer_trait_named(tcx: TyCtxt<'_>, name: &str) -> Option<DefId> {
    let mut found = None;
    for trait_did in tcx.all_traits_including_private() {
        if tcx.item_name(trait_did).as_str() != name || !is_consumer_trait(tcx, trait_did) {
            continue;
        }
        if found.replace(trait_did).is_some() {
            return None;
        }
    }
    found
}

/// Whether a type named `name` is declared where the crate could import it from: in any module of
/// the crate, or public in a direct dependency. A wiring name that does not resolve but names such
/// a type was not imported, rather than never declared.
pub fn is_type_declared(tcx: TyCtxt<'_>, name: &str) -> bool {
    let local = tcx.hir_crate_items(()).definitions().any(|local| {
        is_type_kind(tcx.def_kind(local)) && tcx.item_name(local.to_def_id()).as_str() == name
    });
    local
        || public_dependency_path(tcx, |kind, _, child_name| {
            is_type_kind(kind) && child_name.as_str() == name
        })
        .is_some()
}

/// Whether an item of `kind` is a type a path can name.
fn is_type_kind(kind: DefKind) -> bool {
    matches!(
        kind,
        DefKind::Struct | DefKind::Enum | DefKind::Union | DefKind::TyAlias | DefKind::ForeignTy
    )
}
