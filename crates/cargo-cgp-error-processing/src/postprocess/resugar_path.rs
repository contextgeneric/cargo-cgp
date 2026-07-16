//! Post-processor: resugar the `Path!` type-level path.
//!
//! `Path!(@app.GreeterComponent)` expands to a right-nested `PathCons` chain terminated by
//! `Nil` — `PathCons<Symbol!("app"), PathCons<GreeterComponent, Nil>>` (see the CGP `Path!`
//! reference). This reverses that spine back to `Path!(@app.GreeterComponent)` in the
//! diagnostic text.
//!
//! Like every resugaring post-processor, it rewrites **only on an exact, round-trippable
//! match**. `Path!` classifies each segment the same way going forward: a single identifier
//! whose first character is ASCII-lowercase and that is not a primitive type becomes a
//! `Symbol`; every other segment (a capitalized type, a primitive) is kept as the named
//! type. So a `Symbol!("name")` head is resugared to the bare segment `name` only when `name`
//! is such a lowercase, non-primitive identifier, and a named-type head is kept only when it
//! is a plain identifier `Path!` would leave as a type — a capitalized name or a primitive. A
//! `PathCons` whose spine or segments do not fit is left untouched, because rewriting it could
//! change what it means. A named-type segment may be printed **module-qualified**
//! (`finance::QuantityTypeProviderComponent`) in a multi-module crate; its final component is
//! used, since that is the bare name `Path!` writes, while a segment carrying generics or other
//! non-identifier shape still declines.
//!
//! A path may also be **open-ended**: instead of terminating in `Nil`, its spine ends in a
//! generic "rest of path" parameter, which rustc renders as the inference placeholder `_` in
//! the trait references it prints (as in the conflicting-wiring `E0119` blocks over a
//! duplicated `@`-path key). Such a tail is resugared to a trailing `.*` wildcard segment —
//! `PathCons<Symbol!("foo"), PathCons<Symbol!("bar"), _>>` becomes `Path!(@foo.bar.*)`. The
//! `.*` is not real `Path!` syntax and would not parse back, but it reads far better than the
//! raw spine and marks the path as matching any continuation. Only a bare `_` tail triggers
//! this: `_` is never a concrete path segment, so it is the unambiguous signal of a generic
//! placeholder, and any other non-`Nil` tail still declines.
//!
//! Runs after [`resugar_symbol`](super::resugar_symbol), so a symbol segment is already in its
//! `Symbol!("…")` surface form by the time this matches it.

/// Rewrite every well-formed `PathCons<…>` spine in `text` to its `Path!(@…)` surface form,
/// returning the rewritten text when any was rewritten (and `None` otherwise). A `PathCons<`
/// that does not parse as an exact, round-trippable path is emitted unchanged and scanning
/// resumes after it.
pub fn resugar_path(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut changed = false;

    while let Some(index) = rest.find("PathCons<") {
        out.push_str(&rest[..index]);
        let candidate = &rest[index..];
        match parse_path(candidate) {
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
/// not round-trip through `Path!`. An open `_` tail renders as a trailing `.*` wildcard.
fn parse_path(input: &str) -> Option<(String, usize)> {
    let after_open = input.strip_prefix("PathCons<")?;
    let (head, tail, inner_len) = scan_head_tail(after_open)?;
    let consumed = "PathCons<".len() + inner_len;

    let mut segments = vec![head.trim().to_owned()];
    let mut tail = tail.trim();
    let mut open_tailed = false;
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 256 {
            return None;
        }
        if tail == "Nil" {
            break;
        }
        // An open-ended path ends not in `Nil` but in a generic "rest of path" parameter,
        // which rustc renders as `_`. Record it and stop; it becomes a trailing `.*` below.
        if is_open_tail(tail) {
            open_tailed = true;
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

    let mut rendered = String::from("Path!(@");
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            rendered.push('.');
        }
        rendered.push_str(&render_segment(segment)?);
    }
    // Append the wildcard for an open tail. Every path has at least the head segment before
    // it, so the separating dot is always warranted.
    if open_tailed {
        rendered.push_str(".*");
    }
    rendered.push(')');
    Some((rendered, consumed))
}

/// Whether `tail` is an open-ended path's terminating generic parameter rather than `Nil` or
/// another `PathCons`. rustc prints a free "rest of path" impl parameter as the inference
/// placeholder `_`, and `_` is never a concrete path segment, so a bare `_` is the exact,
/// round-trippable signal of an open tail — no other tail is treated as a wildcard.
fn is_open_tail(tail: &str) -> bool {
    tail == "_"
}

/// Split the content after a `PathCons<` into its `Head` and `Tail`, returning them and the
/// byte offset just past the matching `>`. Angle, paren, and bracket nesting is balanced and
/// string literals are skipped, so a comma or `>` inside a segment does not mislead the scan.
fn scan_head_tail(s: &str) -> Option<(&str, &str, usize)> {
    let mut depth: i32 = 0;
    let mut comma: Option<usize> = None;
    let mut in_string = false;
    let mut escaped = false;

    for (index, c) in s.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '<' | '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '>' if depth == 0 => {
                let comma = comma?;
                return Some((&s[..comma], &s[comma + 1..index], index + 1));
            }
            '>' => depth -= 1,
            ',' if depth == 0 && comma.is_none() => comma = Some(index),
            _ => {}
        }
    }
    None
}

/// Render one spine segment back to its `Path!` surface form, or `None` when it would not
/// round-trip. A `Symbol!("name")` becomes the bare `name` when `name` is a lowercase,
/// non-primitive identifier; a named-type segment is kept as its final identifier when that is
/// capitalized or a primitive.
///
/// The type segment may arrive **module-qualified** — rustc prints a component defined in a
/// sub-module as `finance::QuantityTypeProviderComponent`, not the bare name the user wrote in
/// the `Path!` — so we accept a `::`-path of plain identifiers and render only its final
/// component, matching how `Path!` writes the segment. This is what keeps a multi-module
/// project's path readable rather than a raw `PathCons<…>` spine.
fn render_segment(segment: &str) -> Option<String> {
    let segment = segment.trim();
    if let Some(name) = symbol_inner(segment) {
        let lowercase_ident = is_ident(&name)
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && !is_primitive_type(&name);
        return lowercase_ident.then_some(name);
    }
    let name = type_segment_tail(segment)?;
    (name.starts_with(|c: char| c.is_ascii_uppercase()) || is_primitive_type(name))
        .then(|| name.to_owned())
}

/// The final `::`-separated component of a named-type segment, or `None` unless every
/// component is a plain identifier. A bare identifier (no `::`) returns itself; a
/// module-qualified path (`a::b::C`) returns its tail (`C`); anything carrying generics,
/// references, or other non-identifier shape declines, so only a genuine type name is folded.
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

/// Whether `s` is a single Rust identifier (ASCII letters, digits, and underscores, not
/// starting with a digit).
fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Whether `ident` names a primitive type, matching the `Path!` macro's own rule
/// (`cgp-macro-core`'s `path_element::is_primitive_type`): an `i`/`u`/`f` followed by digits,
/// or one of `char`/`bool`/`usize`/`isize`/`str`. `ident` is assumed to be an ASCII
/// identifier, so byte-slicing its tail is sound.
fn is_primitive_type(ident: &str) -> bool {
    if (ident.starts_with('i') || ident.starts_with('u') || ident.starts_with('f'))
        && ident[1..].chars().all(|c| c.is_numeric())
    {
        return true;
    }
    matches!(ident, "char" | "bool" | "usize" | "isize" | "str")
}
