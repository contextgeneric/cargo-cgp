//! The anchors' decisions over source positions, on plain byte ranges.
//!
//! The driver converts each `Span` at the call. The comparisons follow `rustc_span`'s `Span::contains`,
//! `Span::overlaps`, and `Span::overlaps_or_adjacent`, which read only the byte offsets, so an
//! expansion context never decides them. `Span` equality also compares the context, so a caller that
//! needs it passes the result in ([`Operand::is_caret`]).

/// A half-open range of source bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub lo: u32,
    pub hi: u32,
}

impl ByteRange {
    /// Whether every byte of `other` lies in this range. An empty `other` needs only its position
    /// to lie within the range, ends included.
    pub fn contains(self, other: ByteRange) -> bool {
        self.lo <= other.lo && other.hi <= self.hi
    }

    /// Whether the two ranges share a byte, or an empty range lies strictly inside the other.
    pub fn overlaps(self, other: ByteRange) -> bool {
        self.lo < other.hi && other.lo < self.hi
    }

    /// Whether the two ranges share a byte or touch end to end.
    pub fn overlaps_or_adjacent(self, other: ByteRange) -> bool {
        self.lo <= other.hi && other.lo <= self.hi
    }
}

/// A call's receiver or argument, as [`is_about_call`] reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operand {
    pub range: ByteRange,
    /// Whether the operand's span is the caret itself, its expansion context included. A
    /// macro-expanded operand can have the caret's bytes and still be another span.
    pub is_caret: bool,
}

/// Whether a diagnostic's primary `caret` is about the call spanning `call`: it overlaps the call and
/// does not lie strictly inside one of its `operands` (a method call's receiver and arguments, or a
/// path call's arguments). A caret that is a whole argument (the `PhantomData` tag a failing handler
/// call is reported on) is about the call.
pub fn is_about_call(
    caret: ByteRange,
    call: ByteRange,
    operands: impl IntoIterator<Item = Operand>,
) -> bool {
    caret.overlaps(call)
        && !operands
            .into_iter()
            .any(|operand| operand.range.contains(caret) && !operand.is_caret)
}

/// Where a caret sits relative to a trait impl.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaretPlace {
    /// Outside the impl, so it says nothing about it.
    OutsideImpl,
    /// On the impl outside every method body: its header or a signature.
    ImplHeader,
    /// Inside a method body, where it is about the impl only on a forwarding call.
    MethodBody,
}

/// The place of `caret` in the impl spanning `impl_range`, whose method bodies span `bodies`.
pub fn caret_place(caret: ByteRange, impl_range: ByteRange, bodies: &[ByteRange]) -> CaretPlace {
    if !impl_range.contains(caret) {
        CaretPlace::OutsideImpl
    } else if bodies.iter().any(|body| body.contains(caret)) {
        CaretPlace::MethodBody
    } else {
        CaretPlace::ImplHeader
    }
}
