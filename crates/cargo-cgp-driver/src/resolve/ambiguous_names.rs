//! Which qualified type names are ambiguous, for the path strip to leave qualified.

use cargo_cgp_error_processing::AmbiguousNames;
use rustc_hir::def::{Namespace, Res};
use rustc_hir::{ItemKind, Node, UseKind};
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::print::with_no_trimmed_paths;
use rustc_span::Symbol;
use rustc_span::def_id::{DefId, LocalDefId};

/// The names the crate glob-imports (the resolver's `glob_map`), each with the paths of the items
/// its globs bring in under that name. With trimmed paths rustc qualifies a name only when it is not
/// unique or glob-imported (`trimmed_def_paths` in `rustc_middle/src/ty/print/pretty.rs`), so a
/// qualified name outside the map is ambiguous, and one inside it names a different item unless the
/// printed path is one of the glob's own. Each item's path is printed as rustc prints a name it does
/// not trim. Reading the map and the modules' children computes nothing under the emitter. Forcing
/// `trimmed_def_paths` itself would, and it takes the diagnostic context's lock, which the emitter
/// holds.
///
/// The glob items are read only when `read_glob_items` is set, since a glob's target module comes
/// from the HIR, which the caller must know is already built: reading it under the emitter during
/// name resolution would force lowering, and lowering emits diagnostics through the lock the emitter
/// holds. Without them each glob-imported name keeps no qualifier.
///
/// `None` until name resolution has finished, which the frozen crate store marks (see
/// [`crate_uses_cgp`](super::crate_uses_cgp)).
pub fn ambiguous_type_names(tcx: TyCtxt<'_>, read_glob_items: bool) -> Option<AmbiguousNames> {
    if !tcx.untracked().cstore.is_frozen() {
        return None;
    }
    let mut names = Vec::new();
    for (&import, symbols) in &tcx.resolutions(()).glob_map {
        let module = read_glob_items
            .then(|| glob_target_module(tcx, import))
            .flatten();
        for &symbol in symbols {
            let paths = module
                .map(|module| glob_item_paths(tcx, module, symbol))
                .unwrap_or_default();
            names.push((symbol.to_string(), paths));
        }
    }
    Some(AmbiguousNames::new(names))
}

/// The module (or enum, or trait) the glob `use` item `import` imports from, read off its HIR path.
fn glob_target_module(tcx: TyCtxt<'_>, import: LocalDefId) -> Option<DefId> {
    let Node::Item(item) = tcx.hir_node_by_def_id(import) else {
        return None;
    };
    let ItemKind::Use(tree) = item.kind else {
        return None;
    };
    if !matches!(tree.kind, UseKind::Glob) {
        return None;
    }
    match tree.prefix.res.type_ns? {
        Res::Def(_, did) => Some(did),
        _ => None,
    }
}

/// The printed paths of the type-namespace items `module` exports under `symbol`, which a glob
/// import of `module` brings in.
fn glob_item_paths(tcx: TyCtxt<'_>, module: DefId, symbol: Symbol) -> Vec<String> {
    let children = match module.as_local() {
        Some(local) => tcx.module_children_local(local),
        None => tcx.module_children(module),
    };
    children
        .iter()
        .filter(|child| child.ident.name == symbol)
        .filter_map(|child| match child.res {
            Res::Def(kind, did) if kind.ns() == Some(Namespace::TypeNS) => {
                Some(with_no_trimmed_paths!(tcx.def_path_str(did)))
            }
            _ => None,
        })
        .collect()
}
