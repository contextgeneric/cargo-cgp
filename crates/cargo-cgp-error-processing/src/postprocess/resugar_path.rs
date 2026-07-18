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
//! `&Coord`, `DateTime<Utc>`). Two shapes still decline, leaving the raw `PathCons` spine rather than
//! risk mangling it: a **module-qualified** segment — the [module strip](super::strip_modules) runs
//! first and removes any qualifier, so a residual `::` means the spine is not the bare form `Path!`
//! writes — and a **bare lowercase identifier**, which `Path!` would have made a `Symbol`, so meeting
//! one as a plain type is ambiguous.
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
/// not round-trip through `Path!`. An open `_` tail renders as a trailing `.*` wildcard.
fn parse_path(input: &str, wrap: bool) -> Option<(String, usize)> {
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

    // A rewrite shows the bare `@…` path; the resugaring fallback wraps it as the `Path!(@…)`
    // macro form.
    let mut rendered = String::from(if wrap { "Path!(@" } else { "@" });
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
/// non-primitive identifier. Every other segment was kept verbatim as a type by `Path!` going
/// forward — a capitalized component marker (`GreeterComponent`), a primitive (`u32`), or a compound
/// value type an `open` statement dispatches on (`Vec<u8>`, `&Coord`, `DateTime<Utc>`) — so it is
/// rendered verbatim.
///
/// Two non-Symbol shapes still decline, so the whole path is left as its raw spine rather than
/// silently mangled. A **`::`-qualified** segment declines: the [module strip](super::strip_modules)
/// runs before this stage and removes any qualifier, so a residual `::` (as when this is called
/// directly, not through the chain) means the spine is not the bare form `Path!` writes. A **bare
/// lowercase identifier** declines too: `Path!` turns a lowercase identifier into a `Symbol`, so
/// meeting one as a plain type — rather than inside a `Symbol!(…)` — is ambiguous, not a clean path.
fn render_segment(segment: &str) -> Option<String> {
    let segment = segment.trim();
    if let Some(name) = symbol_inner(segment) {
        let lowercase_ident = is_ident(&name)
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && !is_primitive_type(&name);
        return lowercase_ident.then_some(name);
    }
    // A **module-qualified** segment (`finance::QuantityTypeProviderComponent`) — as rustc prints a
    // component defined in a sub-module when this is called directly rather than after the module
    // strip — keeps only its final component, and only when every part is a plain identifier: a
    // qualified segment whose tail is lowercase (would be a `Symbol`) or carries generics is not the
    // bare form `Path!` writes, so it declines.
    if segment.contains("::") {
        let tail = type_segment_tail(segment)?;
        return (tail.starts_with(|c: char| c.is_ascii_uppercase()) || is_primitive_type(tail))
            .then(|| tail.to_owned());
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
