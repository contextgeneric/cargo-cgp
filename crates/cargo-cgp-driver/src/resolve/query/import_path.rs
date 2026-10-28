//! The path a `use` imports an item by, read off the crates' public module trees.

use std::collections::{HashSet, VecDeque};

use rustc_hir::def::{DefKind, Res};
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::print::{with_crate_prefix, with_no_trimmed_paths};
use rustc_span::Symbol;
use rustc_span::def_id::DefId;

/// How deep the public module tree of a dependency is searched for an item's import path.
const IMPORT_PATH_DEPTH: usize = 6;

/// The path a `use` imports `def_id` by: a `crate::` path for a local item, and for an item of a
/// dependency the shortest public path from a direct dependency's root, following re-exports
/// (`cgp::core::error::ErrorTypeProviderComponent`). The compiler's own path printing names a
/// re-exported crate by its crate name (`cgp::cgp_core::…`), which does not compile where the
/// re-export renames it, so the module tree is walked instead. `None` when no public path is found.
pub fn import_path(tcx: TyCtxt<'_>, def_id: DefId) -> Option<String> {
    if def_id.is_local() {
        return Some(with_no_trimmed_paths!(with_crate_prefix!(
            tcx.def_path_str(def_id)
        )));
    }
    let path = public_dependency_path(tcx, |_, child_did, _| child_did == def_id)?;
    let segments: Vec<&str> = path.iter().map(Symbol::as_str).collect();
    Some(segments.join("::"))
}

/// The shortest public path, from a direct dependency's root and following re-exports, to an item
/// `wanted` accepts by its kind, its `DefId`, and the name it is exported under.
pub fn public_dependency_path(
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
