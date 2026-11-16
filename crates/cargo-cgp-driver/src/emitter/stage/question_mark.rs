//! The stage for a `?`-operator cascade of a CGP failure already reported.

use rustc_errors::DiagInner;
use rustc_span::Span;

use crate::emitter::edit::is_question_mark_cascade;
use crate::emitter::pipeline::Verdict;

/// Drop a `?`-operator error on an expression where a CGP wiring failure was already reported (a
/// primary span overlapping one of `cgp_spans`). A cascade that arrives before its failure is
/// buffered as a pending entry, which the flush drops on the same test
/// ([`settle_pending`](crate::emitter::settle_pending)).
///
/// See cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md for why only the
/// `?` cascade is dropped.
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
