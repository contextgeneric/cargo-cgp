//! The path a `use` imports an item by, read off the crates' public module trees.

use std::collections::{HashMap, HashSet, VecDeque};

use rustc_hir::def::{DefKind, Res};
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::print::{with_crate_prefix, with_no_trimmed_paths};
use rustc_span::Symbol;
use rustc_span::def_id::{CRATE_DEF_ID, DefId, LocalDefId};

use crate::config::CGP_MACRO_PRELUDE_MODULE;

/// How deep the public module tree of a dependency is searched for an item's import path.
const IMPORT_PATH_DEPTH: usize = 6;

/// The path a `use` imports `def_id` by: a `crate::` path for a local item, and for an item of a
/// dependency the shortest public path from a direct dependency's root, following re-exports
/// (`cgp::core::error::ErrorTypeProviderComponent`), never through [`CGP_MACRO_PRELUDE_MODULE`].
/// The compiler's own path printing names a re-exported crate by its crate name
/// (`cgp::cgp_core::…`), which does not compile where the re-export renames it, so the module tree
/// is walked instead. `None` when no public path is found.
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

/// Whether exactly one path imports `def_id` into the module `from`, so an import of it leaves the
/// reader nothing to choose. The paths counted are those through the crate's own modules that
/// `from` can see, and the public paths from a direct dependency's root, following re-exports, as
/// far as [`import_path`] searches. A module two paths reach counts every item below it twice,
/// which is enough to tell one path from several, so each module is searched at most twice.
pub fn has_one_import_path(tcx: TyCtxt<'_>, def_id: DefId, from: LocalDefId) -> bool {
    let dependency_roots = tcx
        .crates(())
        .iter()
        .filter(|&&cnum| {
            tcx.extern_crate(cnum)
                .is_some_and(|krate| krate.is_direct())
        })
        .map(|&cnum| cnum.as_def_id());
    let mut queue: VecDeque<(DefId, Vec<DefId>)> = std::iter::once(CRATE_DEF_ID.to_def_id())
        .chain(dependency_roots)
        .map(|root| (root, vec![root]))
        .collect();
    let mut searched: HashMap<DefId, usize> = HashMap::new();
    let mut found = 0;
    while let Some((module, modules)) = queue.pop_front() {
        let times = searched.entry(module).or_default();
        if modules.len() > IMPORT_PATH_DEPTH || *times == 2 {
            continue;
        }
        *times += 1;
        let children = match module.as_local() {
            Some(local) => tcx.module_children_local(local),
            None => tcx.module_children(module),
        };
        for child in children {
            let visible = match module.as_local() {
                Some(_) => child.vis.is_accessible_from(from, tcx),
                None => child.vis.is_public(),
            };
            let Res::Def(kind, child_did) = child.res else {
                continue;
            };
            if !visible || child.ident.name.as_str() == CGP_MACRO_PRELUDE_MODULE {
                continue;
            }
            if child_did == def_id {
                found += 1;
                if found > 1 {
                    return false;
                }
            } else if kind == DefKind::Mod && !modules.contains(&child_did) {
                let mut child_modules = modules.clone();
                child_modules.push(child_did);
                queue.push_back((child_did, child_modules));
            }
        }
    }
    found == 1
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
            if child.ident.name.as_str() == CGP_MACRO_PRELUDE_MODULE {
                continue;
            }
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
