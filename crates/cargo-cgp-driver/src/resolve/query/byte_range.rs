//! Converting a `Span` to the rustc-free [`ByteRange`] the span decisions read.

use cargo_cgp_error_processing::byte_range::ByteRange;
use rustc_span::Span;

/// The byte range `span` covers, its expansion context dropped.
pub fn byte_range(span: Span) -> ByteRange {
    ByteRange {
        lo: span.lo().0,
        hi: span.hi().0,
    }
}

/// Whether two spans cover the same source range, whatever their expansion contexts. The two halves
/// of an `E0119` pair are re-spanned onto one entry token but carry different contexts, so matching a
/// caret to a generated impl compares ranges, not whole spans.
pub fn same_range(a: Span, b: Span) -> bool {
    byte_range(a) == byte_range(b)
}
