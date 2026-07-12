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
//! change what it means.
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
/// is not `PathCons`/`Nil` all the way down or whose segments do not round-trip through
/// `Path!`.
fn parse_path(input: &str) -> Option<(String, usize)> {
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
    rendered.push(')');
    Some((rendered, consumed))
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
/// non-primitive identifier; a plain identifier that is capitalized or a primitive is kept as
/// the named type.
fn render_segment(segment: &str) -> Option<String> {
    let segment = segment.trim();
    if let Some(name) = symbol_inner(segment) {
        let lowercase_ident = is_ident(&name)
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && !is_primitive_type(&name);
        return lowercase_ident.then_some(name);
    }
    if is_ident(segment)
        && (segment.starts_with(|c: char| c.is_ascii_uppercase()) || is_primitive_type(segment))
    {
        return Some(segment.to_owned());
    }
    None
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
