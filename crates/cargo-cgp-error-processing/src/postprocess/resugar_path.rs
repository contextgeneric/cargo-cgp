//! Post-processor: resugar the `Path!` type-level path, on a well-formed match only.
//!
//! Runs after [`resugar_symbol`](fn@super::resugar_symbol). See
//! cgp-knowledge-base/cargo-cgp/implementation/resugaring.md for how each segment renders, which
//! ones decline, and the open-ended tail.

use crate::postprocess::scan_head_tail;
use crate::syntax::{is_ident, is_primitive_type};

/// Rewrite every well-formed `PathCons<…>` spine in `text` to its surface path form, returning
/// the rewritten text when any was rewritten (and `None` otherwise). A `PathCons<` that does not
/// parse as an exact, round-trippable path is emitted unchanged and scanning resumes after it.
///
/// `wrap` chooses the form. A **rewrite** (a message the tool constructs, such as a typed
/// resolution note or a coded header) wants the bare `@app.GreeterComponent`, since it reads as a
/// path the message names, not a macro call. The **resugaring fallback** (an un-rewritten message
/// where a raw type is being shown back in source form) wants the `Path!(@app.GreeterComponent)`
/// macro form. So `wrap` is `false` in the rewrite paths and `true` in the fallback.
pub fn resugar_path(text: &str, wrap: bool) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut changed = false;

    while let Some(index) = rest.find("PathCons<") {
        out.push_str(&rest[..index]);
        let candidate = &rest[index..];
        match parse_path(candidate, wrap) {
            Some((rendered, consumed)) => {
                out.push_str(&rendered);
                rest = &candidate[consumed..];
                changed = true;
            }
            None => {
                out.push_str("PathCons<");
                rest = &candidate["PathCons<".len()..];
            }
        }
    }

    out.push_str(rest);
    changed.then_some(out)
}

/// Parse a `PathCons<…>` chain at the start of `input` (which begins with `PathCons<`) and
/// render it as `Path!(@…)`, returning that and the bytes consumed. `None` on any spine that
/// is not `PathCons` all the way down to a `Nil` or an open `_` tail, or whose segments do
/// not round-trip through `Path!`. An open `_` tail is left out of the bare form, and declines the
/// `Path!(…)` form.
fn parse_path(input: &str, wrap: bool) -> Option<(String, usize)> {
    let after_open = input.strip_prefix("PathCons<")?;
    let (head, tail, inner_len) = scan_head_tail(after_open)?;
    let consumed = "PathCons<".len() + inner_len;

    let mut segments = vec![head.trim().to_owned()];
    let mut tail = tail.trim();
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 256 {
            return None;
        }
        if tail == "Nil" {
            break;
        }
        // An open-ended path ends not in `Nil` but in the macros' generic "rest of path"
        // parameter, which rustc renders as `_`: left out of the bare key form, while the
        // `Path!(…)` form would name a different, closed path.
        if is_open_tail(tail) {
            if wrap {
                return None;
            }
            break;
        }
        let tail_after_open = tail.strip_prefix("PathCons<")?;
        let (tail_head, tail_tail, tail_inner_len) = scan_head_tail(tail_after_open)?;
        // A tail segment's `PathCons<…>` must be the whole tail: nothing may trail it, or
        // the spine is malformed and must not be silently rewritten.
        if "PathCons<".len() + tail_inner_len != tail.len() {
            return None;
        }
        segments.push(tail_head.trim().to_owned());
        tail = tail_tail.trim();
    }

    // A rewrite shows the bare `@…` path; the resugaring fallback wraps it as the `Path!(@…)`
    // macro form.
    let mut rendered = String::from(if wrap { "Path!(@" } else { "@" });
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            rendered.push('.');
        }
        rendered.push_str(&render_segment(segment)?);
    }
    if wrap {
        rendered.push(')');
    }
    Some((rendered, consumed))
}

/// Whether `tail` is an open-ended path's terminating generic parameter rather than `Nil` or
/// another `PathCons`. rustc prints a free "rest of path" impl parameter as the inference
/// placeholder `_`, and `_` is never a concrete path segment, so a bare `_` is the exact,
/// round-trippable signal of an open tail: no other tail is treated as a wildcard.
fn is_open_tail(tail: &str) -> bool {
    tail == "_"
}

/// Render one spine segment back to its `Path!` surface form, or `None` when it would not
/// round-trip.
///
/// See cgp-knowledge-base/cargo-cgp/implementation/resugaring.md for the segment rules.
fn render_segment(segment: &str) -> Option<String> {
    let segment = segment.trim();
    if let Some(name) = symbol_inner(segment) {
        let lowercase_ident = is_ident(&name)
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && !is_primitive_type(&name);
        return lowercase_ident.then_some(name);
    }
    // A **module-qualified** segment (`std::io::Error`) is a type `Path!` accepts as written, so it
    // stays qualified: rustc prints a name qualified only when it is ambiguous, and the chain
    // strips CGP's own qualifiers before this stage. It must be a run of plain identifiers ending
    // in a type name; a lowercase tail (would be a `Symbol`) or one carrying generics declines.
    // Only the segment's own head counts: a path inside its arguments (`Wrap<a::Item>`) leaves it a
    // compound type.
    let head = segment.split('<').next().unwrap_or(segment);
    if head.contains("::") {
        let tail = type_segment_tail(segment)?;
        return (tail.starts_with(|c: char| c.is_ascii_uppercase()) || is_primitive_type(tail))
            .then(|| segment.to_owned());
    }
    // A bare lowercase identifier would have been a `Symbol`, so as a plain type it is ambiguous.
    if is_ident(segment)
        && segment.starts_with(|c: char| c.is_ascii_lowercase())
        && !is_primitive_type(segment)
    {
        return None;
    }
    // Everything else is kept verbatim, as `Path!` kept it going forward: a capitalized component
    // marker, a primitive, or a compound value type such as `Vec<u8>` or `&Coord`.
    Some(segment.to_owned())
}

/// The final `::`-separated component of a module-qualified named-type segment, or `None` unless
/// every component is a plain identifier. `a::b::C` returns its tail (`C`); a component carrying
/// generics, references, or other non-identifier shape declines, so only a genuine qualified type
/// name is rendered.
fn type_segment_tail(segment: &str) -> Option<&str> {
    let mut tail = None;
    for part in segment.split("::") {
        if !is_ident(part) {
            return None;
        }
        tail = Some(part);
    }
    tail
}

/// The string content of a `Symbol!("…")` segment, or `None` if `segment` is not one. Only a
/// plain, unescaped literal is accepted: an escaped or quote-bearing one is left for
/// [`render_segment`] to decline, since it cannot be a path identifier anyway.
fn symbol_inner(segment: &str) -> Option<String> {
    let literal = segment.strip_prefix("Symbol!(")?.strip_suffix(')')?.trim();
    let inner = literal.strip_prefix('"')?.strip_suffix('"')?;
    if inner.contains('"') || inner.contains('\\') {
        return None;
    }
    Some(inner.to_owned())
}
