//! Post-processor: resugar the `Path!` type-level path.
//!
//! `Path!(@app.GreeterComponent)` expands to a right-nested `PathCons` chain terminated by
//! `Nil` — `PathCons<Symbol!("app"), PathCons<GreeterComponent, Nil>>` (see the CGP `Path!`
//! reference). This reverses that spine back to `Path!(@app.GreeterComponent)` in the
//! diagnostic text.
//!
//! Like every resugaring post-processor, it rewrites **only on a well-formed match**. `Path!`
//! classifies each segment the same way going forward: a single identifier whose first character is
//! ASCII-lowercase and that is not a primitive type becomes a `Symbol`; every other segment is kept
//! verbatim as the named type. So a `Symbol!("name")` head is resugared to the bare segment `name`
//! only when `name` is such a lowercase, non-primitive identifier, and every other segment is
//! rendered back verbatim as its type — a capitalized component marker (`GreeterComponent`), a
//! primitive (`u32`), or a compound value type an `open` statement dispatches on (`Vec<u8>`,
//! `&Coord`, `DateTime<Utc>`), including a module-qualified type (`std::io::Error`), which rustc
//! qualifies only when the bare name is ambiguous. Two shapes still decline, leaving the raw
//! `PathCons` spine rather than risk mangling it: a qualified segment that is not a plain type path,
//! and a **bare lowercase identifier**, which `Path!` would have made a `Symbol`, so meeting one as a
//! plain type is ambiguous.
//!
//! A path may also be **open-ended**: instead of terminating in `Nil`, its spine ends in a
//! generic "rest of path" parameter, which rustc renders as the inference placeholder `_` in
//! the trait references it prints (as in the conflicting-wiring `E0119` blocks over a
//! duplicated `@`-path key). That parameter is the `__Wildcard__` tail `cgp`'s macros end every
//! `@`-path key and redirect target with, which the programmer never writes, so the bare form a
//! rewritten message uses leaves it out: `PathCons<Symbol!("foo"), PathCons<Symbol!("bar"), _>>`
//! reads `@foo.bar`, the key as its entry wrote it. The `Path!(…)` form declines on such a tail
//! instead, since `Path!(@foo.bar)` would name the closed, `Nil`-terminated path, a different
//! type. Only a bare `_` tail is open: `_` is never a concrete path segment, and any other
//! non-`Nil` tail still declines.
//!
//! Runs after [`resugar_symbol`](fn@super::resugar_symbol), so a symbol segment is already in its
//! `Symbol!("…")` surface form by the time this matches it.

use crate::postprocess::scan_head_tail;
use crate::syntax::{is_ident, is_primitive_type};

/// Rewrite every well-formed `PathCons<…>` spine in `text` to its surface path form, returning
/// the rewritten text when any was rewritten (and `None` otherwise). A `PathCons<` that does not
/// parse as an exact, round-trippable path is emitted unchanged and scanning resumes after it.
///
/// `wrap` chooses the form. A **rewrite** — a message the tool constructs, such as a typed
/// resolution note or a coded header — wants the bare `@app.GreeterComponent`, since it reads as a
/// path the message names, not a macro call. The **resugaring fallback** — an un-rewritten message
/// where a raw type is being shown back in source form — wants the `Path!(@app.GreeterComponent)`
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
        // A tail segment's `PathCons<…>` must be the whole tail — nothing may trail it, or
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
/// round-trippable signal of an open tail — no other tail is treated as a wildcard.
fn is_open_tail(tail: &str) -> bool {
    tail == "_"
}

/// Render one spine segment back to its `Path!` surface form, or `None` when it would not
/// round-trip. A `Symbol!("name")` becomes the bare `name` when `name` is a lowercase,
/// non-primitive identifier. Every other segment was kept verbatim as a type by `Path!` going
/// forward — a capitalized component marker (`GreeterComponent`), a primitive (`u32`), or a compound
/// value type an `open` statement dispatches on (`Vec<u8>`, `&Coord`, `DateTime<Utc>`) — so it is
/// rendered verbatim.
///
/// A **`::`-qualified** type segment (`std::io::Error`) is rendered as written, since `Path!` parses
/// it as the same type. Two shapes decline, so the whole path is left as its raw spine rather than
/// silently mangled: a qualified segment that is not a plain type path (a lowercase tail, generics),
/// and a **bare lowercase identifier**, which `Path!` would have made a `Symbol`, so meeting one as a
/// plain type (rather than inside a `Symbol!(…)`) is ambiguous, not a clean path.
fn render_segment(segment: &str) -> Option<String> {
    let segment = segment.trim();
    if let Some(name) = symbol_inner(segment) {
        let lowercase_ident = is_ident(&name)
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && !is_primitive_type(&name);
        return lowercase_ident.then_some(name);
    }
    // A **module-qualified** segment (`std::io::Error`) is a type `Path!` accepts as written, so it
    // stays qualified: rustc prints a name qualified only when it is ambiguous, and the chain strips
    // CGP's own qualifiers before this stage. It must be a run of plain identifiers ending in a type
    // name; a lowercase tail (would be a `Symbol`) or one carrying generics declines.
    if segment.contains("::") {
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
    // Everything else — a capitalized component marker, a primitive, or a compound value type
    // (`Vec<u8>`, `&Coord`) — is kept verbatim as `Path!` kept it going forward.
    Some(segment.to_owned())
}

/// The final `::`-separated component of a module-qualified named-type segment, or `None` unless
/// every component is a plain identifier. `a::b::C` returns its tail (`C`); a component carrying
/// generics, references, or other non-identifier shape declines, so only a genuine qualified type
/// name is folded to its tail.
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
/// plain, unescaped literal is accepted — an escaped or quote-bearing one is left for
/// [`render_segment`] to decline, since it cannot be a path identifier anyway.
fn symbol_inner(segment: &str) -> Option<String> {
    let literal = segment.strip_prefix("Symbol!(")?.strip_suffix(')')?.trim();
    let inner = literal.strip_prefix('"')?.strip_suffix('"')?;
    if inner.contains('"') || inner.contains('\\') {
        return None;
    }
    Some(inner.to_owned())
}
