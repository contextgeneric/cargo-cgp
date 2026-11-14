//! The edit that imports a component not in scope, as a fix an editor or `cargo fix` applies.
//!
//! The fix inserts the `use` line where rustc places its own import suggestion
//! (`UsePlacementFinder` in `rustc_resolve`), and keeps the name the error points at unchanged, as
//! a second part. An editor offers a check's fix only on the fix's own spans, and the `use` line is
//! usually far from the name, so the second part puts the fix where the reader looks.

use cargo_cgp_error_processing::component_import_line;
use rustc_hir::ItemKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::{DefId, LocalDefId, LocalModId};
use rustc_span::{BytePos, Span};

use crate::resolve::query::has_one_import_path;

/// An import of a component: the `use` line, where it goes, and the name it fixes.
pub struct ImportFix {
    /// Where the `use` line goes, an empty span at the start of a line.
    pub use_span: Span,
    /// The `use` line, indented as the item it goes before, with its line break, and a blank line
    /// after it in a module without one.
    pub use_line: String,
    /// The name the error points at, which the fix leaves unchanged.
    pub name_span: Span,
    /// The name's text as written.
    pub name_text: String,
    /// Whether one path imports the component, so the import is the only one to choose.
    pub machine_applicable: bool,
}

/// The fix that imports `marker` by `path` into the module holding `owner`, for the name at
/// `name_span`. `None` when the module has no place for a `use` line the program wrote, such as one
/// a macro generated whole, or the name and the place are in different files.
pub fn import_fix(
    tcx: TyCtxt<'_>,
    owner: LocalDefId,
    marker: DefId,
    path: &str,
    name_span: Span,
) -> Option<ImportFix> {
    let module = tcx.parent_module_from_def_id(owner);
    let (item_start, module_has_use) = use_placement(tcx, module)?;
    let (use_span, indent) = line_start(tcx, item_start);
    let source_map = tcx.sess.source_map();
    if !name_span.can_be_used_for_suggestions()
        || source_map.lookup_source_file(use_span.lo()).name
            != source_map.lookup_source_file(name_span.lo()).name
    {
        return None;
    }
    Some(ImportFix {
        use_span,
        use_line: component_import_line(path, module_has_use, &indent),
        name_span,
        name_text: source_map.span_to_snippet(name_span).ok()?,
        machine_applicable: has_one_import_path(tcx, marker, module.to_local_def_id()),
    })
}

/// Where a `use` line goes in `module`, and whether it goes before another `use` line: before the
/// first `use` item the program wrote, as rustc places its own import, or at the start of the
/// module's items when it has none. A `use` item with an attribute is passed over, since the line
/// inserted before it would come between the item and its attribute, which the HIR keeps without
/// a span. A place inside a macro's expansion is no place the program can edit.
fn use_placement(tcx: TyCtxt<'_>, module: LocalModId) -> Option<(Span, bool)> {
    let module_hir = tcx.hir_get_module(module).0;
    let first_use = module_hir
        .item_ids
        .iter()
        .map(|&item_id| tcx.hir_item(item_id))
        .filter(|item| {
            matches!(item.kind, ItemKind::Use(..))
                && !item.span.from_expansion()
                && tcx.hir_attrs(item.hir_id()).is_empty()
        })
        .map(|item| item.span.shrink_to_lo())
        .min_by_key(|span| span.lo());
    match first_use {
        Some(span) => Some((span, true)),
        None => {
            let inject = module_hir.spans.inject_use_span;
            (!inject.from_expansion()).then_some((inject, false))
        }
    }
}

/// Where a line inserted before the item starting at `item_start` goes, and the indentation it
/// takes: the start of the item's line when only whitespace precedes the item there, so the item
/// keeps its own line and indentation, or the item's start itself, unindented, when other code
/// precedes it on the line.
fn line_start(tcx: TyCtxt<'_>, item_start: Span) -> (Span, String) {
    let position = tcx.sess.source_map().lookup_char_pos(item_start.lo());
    let prefix: String = position
        .file
        .get_line(position.line - 1)
        .map(|line| line.chars().take(position.col.0).collect())
        .unwrap_or_default();
    if prefix.chars().all(|c| c == ' ' || c == '\t') {
        let line_start = item_start.lo() - BytePos(prefix.len() as u32);
        (item_start.with_lo(line_start).shrink_to_lo(), prefix)
    } else {
        (item_start, String::new())
    }
}
