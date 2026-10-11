//! Whether a wiring overflow is a cycle in the wiring.

use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::resolve::anchor::check_entry_obligation;
use crate::resolve::cache::ResolveCache;
use crate::resolve::walk::walk_is_cut;

/// Whether the `E0275` overflow at the `check_components!` entry at `primary_span` comes from a
/// lookup that never terminates in the wiring: walking the entry's consumer obligation meets a
/// cycle or the depth cap. An overflow can also follow from a type the compiler could not resolve,
/// whose walk ends without a cut, and that is no cycle. `None` when no check entry is at the span.
pub fn overflow_is_wiring_cycle(
    tcx: TyCtxt<'_>,
    cache: &ResolveCache,
    primary_span: Span,
) -> Option<bool> {
    let obligation = check_entry_obligation(tcx, primary_span)?;
    Some(walk_is_cut(tcx, cache, obligation))
}
