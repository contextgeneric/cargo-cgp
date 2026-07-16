//! Post-processor: strip module qualifiers from every path, leaving the bare final segment.
//!
//! rustc prints types and traits fully qualified — `contexts::app::MockApp`,
//! `interfaces::types::QuantityTypeProviderComponent`, `std::cmp::Eq` — which is noise in a CGP
//! diagnostic, where the bare name (`MockApp`, `QuantityTypeProviderComponent`, `Eq`) is what the
//! programmer wrote and reads. This collapses each `a::b::C` run to its last segment `C`, in both
//! the rewritten CGP messages and the resugaring fallback, so the two read the same. It subsumes
//! [`strip_cgp_prefixes`](super::strip_prefixes) (a `cgp::prelude::Chars` is just one such run) and
//! runs first, before the resugaring stages, so they match the bare names.
//!
//! It rewrites **only** a run of plain identifiers joined by `::` — `foo::Bar`, `a::b::c::D` — and
//! leaves everything else alone: a lone identifier keeps its name, a turbofish (`foo::<T>`) is not
//! an identifier run so it is untouched, an associated-type projection's `>::Assoc` tail has no
//! identifier before its `::` so it stays, and text inside a string literal is skipped so a
//! `Symbol!("a::b")` (were one to arise) is never mangled.

/// Strip module qualifiers from `text`, collapsing each `a::b::C` identifier run to its last
/// segment. Returns the rewritten text when any run was collapsed, `None` otherwise.
pub fn strip_module_paths(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut in_string = false;
    let mut changed = false;

    // Identifier runs are pure ASCII, so they are scanned by byte; every other character is copied
    // whole (by its UTF-8 width) rather than byte-by-byte, so multi-byte text — the `└──`
    // box-drawing of a rendered dependency tree, say — is never split into invalid bytes.
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            let ch = next_char(text, i);
            out.push(ch);
            i += ch.len_utf8();
            // Skip an escaped character whole, so a `\"` does not end the string early.
            if ch == '\\' && i < bytes.len() {
                let esc = next_char(text, i);
                out.push(esc);
                i += esc.len_utf8();
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if b == b'"' {
            in_string = true;
            out.push('"');
            i += 1;
            continue;
        }
        if is_ident_start(b) {
            // Walk an identifier run joined by `::`; only the final segment is emitted, dropping
            // the module qualifiers before it.
            let mut j = i;
            loop {
                let seg_start = j;
                while j < bytes.len() && is_ident_char(bytes[j]) {
                    j += 1;
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
                out.push_str(&text[seg_start..j]);
                if seg_start != i {
                    changed = true;
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
