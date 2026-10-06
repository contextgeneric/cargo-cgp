//! Tightening the spacing the printer leaves inside a resugared macro call.
//!
//! A resugared construct is a macro call whose body holds ordinary types, and the printer lays a
//! macro body out token by token: it has no way to know the body is a type list, so it spaces
//! every token apart and prints `Product![Multiply < Symbol!("foo") >]`. Its spacing rules cannot
//! be coaxed into the conventional form — the space *before* a token is the printer's decision and
//! an identifier cannot ask for it to be dropped — so the layout is corrected after printing.
//!
//! The printer also always breaks a brace-delimited macro body onto lines of its own, printing a
//! `Struct! { … }` shape as a block. It still puts the whole body on one line, so a brace body is
//! joined back onto the macro's line, `Struct! { width: f64 }`, the way a diagnostic and the
//! documentation write a shape.
//!
//! This is the one text pass in the crate, and it is deliberately narrow: it only ever removes
//! whitespace (joining a brace body replaces each line break with a single space), only inside the
//! body of one of the CGP macros [`SUGAR_MACROS`] names, and never inside a literal. So it cannot
//! alter the meaning of anything, and it cannot reach the surrounding program at all. (A
//! `Product![…]` a programmer wrote themselves is tightened the same way, which is the same
//! transformation and equally harmless.)

/// How a macro body is laid out after tightening.
#[derive(Clone, Copy)]
enum Layout {
    /// Keep the printer's line breaks, so a long list stays broken across lines.
    AsPrinted,
    /// Join the body onto the macro's line, between single spaces: `Struct! { width: f64 }`.
    Inline,
}

/// The macro calls whose bodies are tightened: the ones this crate emits. A body is located by an
/// exact name match followed by its opening delimiter.
const SUGAR_MACROS: &[(&str, char, char, Layout)] = &[
    ("Symbol!", '(', ')', Layout::AsPrinted),
    ("Path!", '(', ')', Layout::AsPrinted),
    ("Product!", '[', ']', Layout::AsPrinted),
    ("Sum!", '[', ']', Layout::AsPrinted),
    ("Struct!", '(', ')', Layout::AsPrinted),
    ("Struct!", '{', '}', Layout::Inline),
    ("Enum!", '{', '}', Layout::Inline),
];

/// Remove the spaces the printer inserted inside every resugared macro body in `printed`.
pub fn tighten_sugar_bodies(printed: &str) -> String {
    let mut out = String::with_capacity(printed.len());
    let mut rest = printed;

    while let Some((index, open, close, layout)) = next_sugar(rest) {
        let (before, from_macro) = rest.split_at(index);
        out.push_str(before);

        // Copy the name and opening delimiter through, then tighten the balanced body.
        let name_end = from_macro
            .find(open)
            .expect("the delimiter was just matched")
            + 1;
        out.push_str(&from_macro[..name_end]);
        let body_start = &from_macro[name_end..];

        match (body_end(body_start, open, close), layout) {
            (Some(end), Layout::AsPrinted) => {
                out.push_str(&tighten(&body_start[..end]));
                rest = &body_start[end..];
            }
            (Some(end), Layout::Inline) => {
                // `end` is the closing delimiter, which `rest` carries on.
                out.push(' ');
                out.push_str(&tighten(&join_lines(&body_start[..end])));
                out.push(' ');
                rest = &body_start[end..];
            }
            // An unbalanced body cannot be trusted, so it is copied through untouched.
            (None, _) => rest = body_start,
        }
    }

    out.push_str(rest);
    out
}

/// The next resugared macro call in `text`: its offset, its delimiter pair, and its layout. The
/// name must stand alone rather than end a longer identifier, so a `MySum!` is not mistaken for a
/// `Sum!`.
fn next_sugar(text: &str) -> Option<(usize, char, char, Layout)> {
    let mut best: Option<(usize, char, char, Layout)> = None;

    for (name, open, close, layout) in SUGAR_MACROS {
        let mut from = 0;
        while let Some(found) = text[from..].find(name) {
            let index = from + found;
            let preceded_by_ident = text[..index]
                .chars()
                .next_back()
                .is_some_and(|c| c == '_' || c.is_alphanumeric());
            // The printer may break between the name and its delimiter, so allow whitespace.
            let after = text[index + name.len()..].trim_start();
            if !preceded_by_ident && after.starts_with(*open) {
                if best.is_none_or(|(current, ..)| index < current) {
                    best = Some((index, *open, *close, *layout));
                }
                break;
            }
            from = index + name.len();
        }
    }

    best
}

/// The offset of the delimiter that closes a body starting at `text`, or `None` when the
/// delimiters do not balance. Delimiters inside a string or character literal are skipped.
fn body_end(text: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 1usize;
    let mut chars = text.char_indices();

    while let Some((index, c)) = chars.next() {
        match c {
            '"' => skip_string(&mut chars),
            '\'' => skip_char_literal(&mut chars),
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

/// Consume the rest of a string literal, honouring escapes.
fn skip_string(chars: &mut std::str::CharIndices<'_>) {
    while let Some((_, c)) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '"' => return,
            _ => {}
        }
    }
}

/// Consume the rest of a character literal. A lifetime (`'a`) has no closing quote, so the scan
/// stops at anything that cannot be part of one rather than running to the end of the body.
fn skip_char_literal(chars: &mut std::str::CharIndices<'_>) {
    let mut lookahead = chars.clone();
    match lookahead.next() {
        // An escape is always a character literal: consume through the closing quote.
        Some((_, '\\')) => {
            while let Some((_, c)) = chars.next() {
                match c {
                    '\\' => {
                        chars.next();
                    }
                    '\'' => return,
                    _ => {}
                }
            }
        }
        // `'c'` — one character then a quote. Anything else is a lifetime, left to the main scan.
        Some(_) => {
            if let Some((_, '\'')) = lookahead.next() {
                chars.next();
                chars.next();
            }
        }
        None => {}
    }
}

/// A brace body with the printer's line breaks and their indentation collapsed to single spaces,
/// and the space it leaves at either end trimmed. String literals are copied whole.
fn join_lines(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    let mut pending_space = false;

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(c);
        if c == '"' {
            copy_string(&mut chars, &mut out);
        }
    }

    out
}

/// Remove the spaces around a macro body's punctuation, leaving newlines and the space after a
/// comma alone so a body the printer broke across lines keeps its shape.
fn tighten(body: &str) -> String {
    // The characters a space is dropped beside: the generic brackets, a reference, and a path
    // separator. A space before a comma goes too; the one after it stays, as in ordinary Rust.
    // The space after a single `:` also stays, since that `:` separates a `Struct!` field from
    // its type rather than the segments of a path.
    const TIGHT_AFTER: &[char] = &['<', '&', ':'];
    const TIGHT_BEFORE: &[char] = &['<', '>', ',', ':'];

    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                out.push(c);
                copy_string(&mut chars, &mut out);
            }
            ' ' => {
                let previous = out.chars().next_back();
                let next = chars.peek().copied();
                let after_field_colon = previous == Some(':') && !out.ends_with("::");
                let drop_space = (previous.is_some_and(|p| TIGHT_AFTER.contains(&p))
                    && !after_field_colon)
                    || next.is_some_and(|n| TIGHT_BEFORE.contains(&n));
                if !drop_space {
                    out.push(' ');
                }
            }
            _ => out.push(c),
        }
    }

    out
}

/// Copy the rest of a string literal into `out` verbatim, so its contents are never tightened.
fn copy_string(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, out: &mut String) {
    while let Some(c) = chars.next() {
        out.push(c);
        match c {
            '\\' => {
                if let Some(escaped) = chars.next() {
                    out.push(escaped);
                }
            }
            '"' => return,
            _ => {}
        }
    }
}
