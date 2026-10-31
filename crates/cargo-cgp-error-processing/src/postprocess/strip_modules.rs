//! Post-processor: strip module qualifiers from every path, leaving the item the path names.
//!
//! rustc prints types and traits fully qualified — `contexts::app::MockApp`,
//! `interfaces::types::QuantityTypeProviderComponent`, `std::cmp::Eq` — which is noise in a CGP
//! diagnostic, where the bare name (`MockApp`, `QuantityTypeProviderComponent`, `Eq`) is what the
//! programmer wrote and reads. This drops the module segments of each `a::b::C` run, in both
//! the rewritten CGP messages and the resugaring fallback, so the two read the same. A module is
//! named in lowercase and a type or trait in uppercase, so the run keeps everything from its first
//! uppercase segment: an item inside a trait or type keeps its qualifier
//! (`cgp::prelude::TryComputer::try_compute` reads `TryComputer::try_compute`), while a run with
//! no uppercase segment, such as a free function's path, keeps only its last segment. A CGP
//! re-export path such as `cgp::prelude::Chars` is just one such run. The strip runs first, before
//! the resugaring stages, so they match the bare names.
//!
//! It rewrites **only** a run of plain identifiers joined by `::` — `foo::Bar`, `a::b::c::D` — and
//! leaves everything else alone: a lone identifier keeps its name, a turbofish (`foo::<T>`) is not
//! an identifier run so it is untouched, an associated-type projection's `>::Assoc` tail has no
//! identifier before its `::` so it stays, a `use` statement's path keeps every segment since the
//! reader types it as written (`use cgp::core::field::impls::CanDowncastFields;`), and text inside
//! a string literal is skipped so a `Symbol!("a::b")` (were one to arise) is never mangled.

use crate::postprocess::ambiguous::AmbiguousNames;
use crate::syntax::string_literal_end;

/// Strip module qualifiers from `text`, collapsing each `a::b::C` identifier run to its tail from
/// its first uppercase segment, or to its last segment when none is uppercase. Returns the
/// rewritten text when any run was collapsed, `None` otherwise.
pub fn strip_module_paths(text: &str) -> Option<String> {
    collapse_paths(text, |_, _, _| true)
}

/// The CGP type names the resugaring stages and the missing-field rewrite read in bare form, so a
/// path ending in one always collapses, whatever crate re-exports it (`hypershell::prelude::Chars`).
/// Some of them are ambiguous (`std::str::Chars`, `std::ops::Index`), but in a CGP diagnostic they
/// are the encoding the later stages fold away.
pub const CGP_ENCODING_NAMES: &[&str] = &[
    "Symbol", "Chars", "Nil", "Cons", "Either", "Void", "PathCons", "Field", "Index", "HasField",
];

/// Strip module qualifiers from `text` except where they tell two items apart: collapse each
/// `a::b::C` run to `C` unless [`AmbiguousNames::keeps_qualifier`] says the bare `C` would read as
/// another item (`Error` for `std::io::Error` beside `backend::Error`). A run rooted at the `cgp` crate
/// always collapses, since rustc qualifies CGP's names for the prelude glob import, and so does a
/// run ending in one of [`CGP_ENCODING_NAMES`]. Returns the rewritten text when any run was
/// collapsed, `None` otherwise.
pub fn strip_unambiguous_paths(text: &str, ambiguous: &AmbiguousNames) -> Option<String> {
    collapse_paths(text, |path, first_segment, last_segment| {
        first_segment == "cgp"
            || CGP_ENCODING_NAMES.contains(&last_segment)
            || !ambiguous.keeps_qualifier(path, last_segment)
    })
}

/// Collapse each `a::b::C` identifier run that `collapse` accepts, given the path to the item the
/// run names, its first segment, and that item's name, to the run's tail from the item: the run
/// from its first uppercase segment (`TryComputer::try_compute`), or its last segment when no
/// earlier segment is uppercase. String literals and every other run are left alone.
fn collapse_paths(text: &str, collapse: impl Fn(&str, &str, &str) -> bool) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut changed = false;

    // Identifier runs are pure ASCII, so they are scanned by byte; every other character is copied
    // whole (by its UTF-8 width) rather than byte-by-byte, so multi-byte text — the `└─`
    // box-drawing of a rendered dependency tree, say — is never split into invalid bytes.
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' {
            let end = string_literal_end(text, i);
            out.push_str(&text[i..end]);
            i = end;
            continue;
        }
        if is_ident_start(b) {
            // A `use` statement's path is code the reader types, so it keeps every segment.
            let in_use = text[..i]
                .strip_suffix("use ")
                .is_some_and(|before| !before.bytes().last().is_some_and(is_ident_char));
            // Walk an identifier run joined by `::`; only the final segment is emitted, dropping
            // the module qualifiers before it.
            let mut j = i;
            let mut first_end = None;
            // Where the item path begins: the first segment named in uppercase, if any.
            let mut item_start: Option<usize> = None;
            loop {
                let seg_start = j;
                while j < bytes.len() && is_ident_char(bytes[j]) {
                    j += 1;
                }
                let first_end = *first_end.get_or_insert(j);
                if item_start.is_none() && bytes[seg_start].is_ascii_uppercase() {
                    item_start = Some(seg_start);
                }
                // Continue the run only across a `::` that is followed by another identifier —
                // not a turbofish `::<` and not a trailing `::` before a non-identifier.
                if j + 2 < bytes.len()
                    && bytes[j] == b':'
                    && bytes[j + 1] == b':'
                    && is_ident_start(bytes[j + 2])
                {
                    j += 2;
                    continue;
                }
                let tail_start = item_start.unwrap_or(seg_start);
                let name_end = text[tail_start..j]
                    .find("::")
                    .map_or(j, |at| tail_start + at);
                if tail_start != i
                    && !in_use
                    && collapse(
                        &text[i..name_end],
                        &text[i..first_end],
                        &text[tail_start..name_end],
                    )
                {
                    out.push_str(&text[tail_start..j]);
                    changed = true;
                } else {
                    out.push_str(&text[i..j]);
                }
                break;
            }
            i = j;
            continue;
        }
        let ch = next_char(text, i);
        out.push(ch);
        i += ch.len_utf8();
    }

    changed.then_some(out)
}

/// The character beginning at byte offset `i` of `text` (which must be a char boundary).
fn next_char(text: &str, i: usize) -> char {
    text[i..]
        .chars()
        .next()
        .expect("byte offset is a char boundary within the string")
}

/// Whether `b` can start a Rust identifier (ASCII letter or underscore).
fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

/// Whether `b` can continue a Rust identifier (ASCII alphanumeric or underscore).
fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
