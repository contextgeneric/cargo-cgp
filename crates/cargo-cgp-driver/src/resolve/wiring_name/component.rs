//! Looking up components by name across the crate graph.
//!
//! A component is found through its provider trait: `#[cgp_component]` gives every provider trait a
//! delegation blanket bounded on `DelegateComponent<Marker>`, and [`provider_blanket_marker`] reads
//! the marker off it. Scanning every trait the crate can see therefore yields every component,
//! local or from a dependency, with its provider trait.

use std::collections::{HashSet, VecDeque};

use rustc_hir::def::{DefKind, Res};
use rustc_middle::ty::print::{with_crate_prefix, with_no_trimmed_paths};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;
use rustc_span::def_id::DefId;

use crate::resolve::cgp_item::{
    consumer_provider_trait, is_consumer_trait, provider_blanket_marker,
};

/// Every component marker the crate can see, each with its provider trait.
pub(crate) fn component_markers(tcx: TyCtxt<'_>) -> Vec<(DefId, DefId)> {
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
pub(crate) fn consumer_component(tcx: TyCtxt<'_>, consumer_did: DefId) -> Option<DefId> {
    let provider_did = consumer_provider_trait(tcx, consumer_did)?;
    let marker = provider_blanket_marker(tcx, provider_did)?;
    match marker.kind() {
        ty::Adt(marker_def, _) => Some(marker_def.did()),
        _ => None,
    }
}

/// The one consumer trait named `name`, or `None` when there is none or more than one.
pub(crate) fn consumer_trait_named(tcx: TyCtxt<'_>, name: &str) -> Option<DefId> {
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

/// How deep the public module tree of a dependency is searched for an item's import path.
const IMPORT_PATH_DEPTH: usize = 6;

/// The path a `use` imports `def_id` by: a `crate::` path for a local item, and for an item of a
/// dependency the shortest public path from a direct dependency's root, following re-exports
/// (`cgp::core::error::ErrorTypeProviderComponent`). The compiler's own path printing names a
/// re-exported crate by its crate name (`cgp::cgp_core::…`), which does not compile where the
/// re-export renames it, so the module tree is walked instead. `None` when no public path is found.
pub(crate) fn import_path(tcx: TyCtxt<'_>, def_id: DefId) -> Option<String> {
    if def_id.is_local() {
        return Some(with_no_trimmed_paths!(with_crate_prefix!(
            tcx.def_path_str(def_id)
        )));
    }
    let path = public_dependency_path(tcx, |_, child_did, _| child_did == def_id)?;
    let segments: Vec<&str> = path.iter().map(Symbol::as_str).collect();
    Some(segments.join("::"))
}

/// Whether a type named `name` is declared where the crate could import it from: in any module of
/// the crate, or public in a direct dependency. A wiring name that does not resolve but names such
/// a type was not imported, rather than never declared.
pub(crate) fn is_type_declared(tcx: TyCtxt<'_>, name: &str) -> bool {
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

/// The shortest public path, from a direct dependency's root and following re-exports, to an item
/// `wanted` accepts by its kind, its `DefId`, and the name it is exported under.
fn public_dependency_path(
    tcx: TyCtxt<'_>,
    mut wanted: impl FnMut(DefKind, DefId, Symbol) -> bool,
) -> Option<Vec<Symbol>> {
    let mut queue: VecDeque<(DefId, Vec<Symbol>)> = tcx
        .crates(())
        .iter()
        .filter(|&&cnum| {
            tcx.extern_crate(cnum)
                .is_some_and(|krate| krate.is_direct())
        })
        .map(|&cnum| (cnum.as_def_id(), vec![tcx.crate_name(cnum)]))
        .collect();
    let mut visited = HashSet::new();
    while let Some((module, path)) = queue.pop_front() {
        if path.len() > IMPORT_PATH_DEPTH || !visited.insert(module) {
            continue;
        }
        for child in tcx.module_children(module) {
            if !child.vis.is_public() {
                continue;
            }
            let Res::Def(kind, child_did) = child.res else {
                continue;
            };
            let mut child_path = path.clone();
            child_path.push(child.ident.name);
            if wanted(kind, child_did, child.ident.name) {
                return Some(child_path);
            }
            if kind == DefKind::Mod {
                queue.push_back((child_did, child_path));
            }
        }
    }
    None
}
