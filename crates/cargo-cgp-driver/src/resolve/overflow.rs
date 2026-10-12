//! How a wiring overflow never resolves: a cycle in the wiring, or a lookup that grows.

use cargo_cgp_error_processing::WiringOverflow;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::resolve::anchor::check_entry_obligation;
use crate::resolve::cache::ResolveCache;
use crate::resolve::walk::walk_overflow;

/// What walking the `check_components!` entry at `primary_span` shows about its `E0275` overflow.
pub enum OverflowWalk {
    /// No check entry sits at the span, so the walk cannot say why the lookup overflowed.
    NoEntry,
    /// The walk ran to its leaves: the overflow follows from something other than the wiring, such
    /// as a type the compiler could not resolve.
    NotCut,
    /// The walk met an obligation again, or reached its depth cap, as the reason says.
    Cut(WiringOverflow),
}

/// Walk the consumer obligation of the check entry at `primary_span` and report how its lookup
/// never resolves (see [`OverflowWalk`]).
pub fn overflow_walk(tcx: TyCtxt<'_>, cache: &ResolveCache, primary_span: Span) -> OverflowWalk {
    let Some(obligation) = check_entry_obligation(tcx, primary_span) else {
        return OverflowWalk::NoEntry;
    };
    match walk_overflow(tcx, cache, obligation) {
        Some(reason) => OverflowWalk::Cut(reason),
        None => OverflowWalk::NotCut,
    }
}
