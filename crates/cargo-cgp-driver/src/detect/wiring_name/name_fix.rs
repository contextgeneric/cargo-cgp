//! The edits that repair a wiring name, as a fix an editor or `cargo fix` applies.
//!
//! A fix makes its edits and keeps the name the error points at unchanged, as a last part. An
//! editor offers a check's fix only on the fix's own spans, and the edits are often elsewhere, so
//! the unchanged part puts the fix where the reader looks. The import of a component not in scope
//! ([`import_fix`]) inserts its `use` line where rustc places its own import suggestion
//! (`UsePlacementFinder` in `rustc_resolve`); the component named in a provider header's attribute
//! is built beside the header ([`derived_component_fix`](super::derived_component_fix)), with the
//! same `use` line ([`import_edit`]) when the key is not in scope there.

use cargo_cgp_error_processing::{
    UnresolvedWiringName, component_import_line, unresolved_wiring_name_fix_message,
};
use rustc_hir::ItemKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::{DefId, LocalDefId, LocalModId};
use rustc_span::{BytePos, Span};

use crate::resolve::query::has_one_import_path;

/// A fix of a wiring name: its message, the edits, and the name the error points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameFix {
    /// The message, which the edits' text follows.
    pub message: String,
    /// Each edit, as where it goes and the text it writes there.
    pub edits: Vec<(Span, String)>,
    /// The name the error points at, which the fix leaves unchanged.
    pub name_span: Span,
    /// The name's text as written.
    pub name_text: String,
    /// Whether the fix is the one repair to choose, so an editor or `cargo fix` may apply it.
    pub machine_applicable: bool,
}

impl NameFix {
    /// The fix of `name` that makes `edits` and keeps the name at `name_span`, worded with
    /// `imports_component` when one edit imports the component. `None` when `name` has no fix, or
    /// a span is not one the program can edit, or the spans are in different files.
    pub fn new(
        tcx: TyCtxt<'_>,
        name: &UnresolvedWiringName,
        imports_component: bool,
        edits: Vec<(Span, String)>,
        name_span: Span,
        machine_applicable: bool,
    ) -> Option<Self> {
        let source_map = tcx.sess.source_map();
        let file = source_map.lookup_source_file(name_span.lo()).name.clone();
        if !name_span.can_be_used_for_suggestions()
            || edits.iter().any(|(span, _)| {
                span.from_expansion() || source_map.lookup_source_file(span.lo()).name != file
            })
        {
            return None;
        }
        Some(Self {
            message: unresolved_wiring_name_fix_message(name, imports_component)?.to_owned(),
            edits,
            name_span,
            name_text: source_map.span_to_snippet(name_span).ok()?,
            machine_applicable,
        })
    }
}

/// The fix that imports `marker` into the module holding `owner`, for `name`, a component not in
/// scope, at `name_span`, machine-applicable when one path imports it. `None` when the module has
/// no place for a `use` line the program wrote, such as one a macro generated whole.
pub fn import_fix(
    tcx: TyCtxt<'_>,
    owner: LocalDefId,
    marker: DefId,
    name: &UnresolvedWiringName,
    name_span: Span,
) -> Option<NameFix> {
    let UnresolvedWiringName::ComponentNotImported { path, .. } = name else {
        return None;
    };
    let module = tcx.parent_module_from_def_id(owner);
    NameFix::new(
        tcx,
        name,
        true,
        vec![import_edit(tcx, module, path)?],
        name_span,
        has_one_import_path(tcx, marker, module.to_local_def_id()),
    )
}

/// The edit that inserts `use {path};` into `module`, as a line where rustc places its own import.
/// `None` when the module has no place for a `use` line the program wrote.
pub fn import_edit(tcx: TyCtxt<'_>, module: LocalModId, path: &str) -> Option<(Span, String)> {
    let (item_start, module_has_use) = use_placement(tcx, module)?;
    let (use_span, indent) = line_start(tcx, item_start);
    Some((
        use_span,
        component_import_line(path, module_has_use, &indent),
    ))
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
