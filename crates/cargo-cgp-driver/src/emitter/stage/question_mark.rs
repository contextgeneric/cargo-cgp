//! The stage for a `?`-operator cascade of a CGP failure already reported.

use rustc_errors::DiagInner;
use rustc_span::Span;

use crate::emitter::edit::is_question_mark_cascade;
use crate::emitter::pipeline::Verdict;

/// Stage 15: drop a `?`-operator error on an expression where a CGP wiring failure was already
/// reported (a primary span overlapping one of `cgp_spans`). Once a wiring bound on `expr` fails,
/// the type of `expr?` cannot be resolved, so rustc adds a `Try` or `FromResidual` error at the same
/// span that restates the failure and dumps the unresolved projected type. A `?` error with no CGP
/// failure on its expression is left alone. The stage runs before the resolver, which would
/// otherwise re-anchor the cascade onto the failing call it overlaps.
///
/// Only the `?` cascade is dropped, never a `[T]: Sized` one: an unsized error the failed method or
/// trait resolution trails can land off the failing expression (on the binding pattern, or a later
/// statement the unresolved type flows into), where a span-overlap check misses it and a broader
/// check would risk dropping an unrelated error.
pub fn question_mark_cascade(diag: &DiagInner, cgp_spans: &[Span]) -> Verdict {
    let overlaps_cgp_failure = diag
        .span
        .primary_spans()
        .iter()
        .any(|span| cgp_spans.iter().any(|seen| seen.overlaps(*span)));
    if is_question_mark_cascade(diag) && overlaps_cgp_failure {
        Verdict::Drop
    } else {
        Verdict::Continue
    }
}
