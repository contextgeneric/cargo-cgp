//! The canonical order the emitter decides and emits diagnostics in.
//!
//! rustc type-checks items on several threads, so the order diagnostics arrive in varies between
//! runs. The emitter therefore processes and prints them in an order that depends only on each
//! diagnostic: by its primary span, an inner span before the span around it, then by its code, its
//! message, and the rest of its content. The driver reads each key off a diagnostic
//! ([`OrderKey`]); the comparison lives here, so it is tested without a compiler.

use std::cmp::Ordering;

use crate::byte_range::ByteRange;

/// Where a diagnostic's primary span sits: its file's name and its bytes within that file. The
/// offsets are the file's own, never the source map's global positions, since a dependency's file
/// enters the source map whenever a span from it is first decoded, which under threads happens in
/// any order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanPlace {
    /// The file's name, as the compiler displays it.
    pub file: String,
    /// The span's bytes, as offsets from the start of the file.
    pub range: ByteRange,
}

/// A diagnostic's place in the canonical order.
///
/// Two keys compare equal when neither diagnostic has a span, whatever their content, so a stable
/// sort keeps such diagnostics in the order they arrived. rustc's closing lines, such as the "For
/// more information about this error" footer, carry no span and come in a fixed order.
#[derive(Debug, Clone)]
pub struct OrderKey {
    /// The primary span's place, or `None` for a diagnostic without a span.
    pub place: Option<SpanPlace>,
    /// The error code, such as `E0277`.
    pub code: Option<String>,
    /// The main message.
    pub message: String,
    /// The rest of the diagnostic's content, each part as a string that holds no global position:
    /// its span labels, its children, its suggestions, and its message arguments. Two diagnostics
    /// with a span tie only when they print the same, so their order cannot change the output.
    pub detail: Vec<String>,
}

impl Ord for OrderKey {
    /// A diagnostic with a span before one without; then by file name; then an inner span before
    /// an enclosing one and an earlier span before a later one, which ordering by the span's end
    /// and then by its start, reversed, gives at once; then by code, then by message, then by
    /// detail.
    ///
    /// Property: for all spans `a` and `b` in one file, when `b.contains(a)` and `a != b`, the key
    /// at `a` comes first; when `a.hi <= b.lo` and neither contains the other, the key at `a` comes
    /// first.
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.place, &other.place) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a
                .file
                .cmp(&b.file)
                .then(a.range.hi.cmp(&b.range.hi))
                .then(b.range.lo.cmp(&a.range.lo))
                .then_with(|| self.code.cmp(&other.code))
                .then_with(|| self.message.cmp(&other.message))
                .then_with(|| self.detail.cmp(&other.detail)),
        }
    }
}

impl PartialOrd for OrderKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrderKey {
    /// Equal exactly when [`Ord::cmp`] says so, so two diagnostics without a span are equal.
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for OrderKey {}

/// `items` with those `preferred` holds for first, each part keeping its order.
///
/// The emitter processes a batch in this order with `preferred` marking the diagnostics at a
/// `check_components!` entry, so of the copies of one mistake, which the de-duplication ledger
/// keeps the first of, the check's copy is the one kept. Being a check entry is a fact about the
/// diagnostic, so the choice does not depend on the order rustc produced the copies in.
///
/// Postcondition: the result is a permutation of `items` in which every preferred item comes before
/// every other, and two items that are both preferred, or both not, keep their relative order.
pub fn preferred_first<T>(items: Vec<T>, preferred: impl Fn(&T) -> bool) -> Vec<T> {
    let (mut first, rest): (Vec<T>, Vec<T>) = items.into_iter().partition(|item| preferred(item));
    first.extend(rest);
    first
}

/// `items` sorted by the key `key_of` gives each, keeping items with equal keys in their order.
///
/// Postcondition: the result is a permutation of `items`, sorted by `key_of`, and two items with
/// equal keys keep their relative order.
pub fn sort_by_order_key<T>(items: Vec<T>, key_of: impl Fn(&T) -> OrderKey) -> Vec<T> {
    let mut keyed: Vec<(OrderKey, T)> = items
        .into_iter()
        .map(|item| (key_of(&item), item))
        .collect();
    // `sort_by` is stable, so equal keys keep their order.
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    keyed.into_iter().map(|(_, item)| item).collect()
}
