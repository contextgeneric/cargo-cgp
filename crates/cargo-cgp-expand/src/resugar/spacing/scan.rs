//! Finding a resugared macro call's body: a literal- and comment-aware scan.

use std::iter::Peekable;
use std::str::CharIndices;

use crate::resugar::spacing::layout::Layout;

/// The macro calls whose bodies are tightened: the ones this crate emits. A body is located by an
/// exact name match followed by its opening delimiter.
pub const SUGAR_MACROS: &[(&str, char, char, Layout)] = &[
    ("Symbol!", '(', ')', Layout::AsPrinted),
    ("Path!", '(', ')', Layout::AsPrinted),
    ("Product!", '[', ']', Layout::AsPrinted),
    ("Sum!", '[', ']', Layout::AsPrinted),
    ("Struct!", '(', ')', Layout::AsPrinted),
    ("Struct!", '{', '}', Layout::Inline),
    ("Enum!", '{', '}', Layout::Inline),
];

/// The next resugared macro call in `text`: its offset, its delimiter pair, and its layout. The
/// name must stand alone rather than end a longer identifier, so a `MySum!` is not mistaken for a
/// `Sum!`, and it must sit in code: the scan steps over string, raw-string, and character literals
/// and over line comments (the printer writes every doc comment as one, and drops the rest), so
/// text that merely names a macro is never edited.
pub fn next_sugar(text: &str) -> Option<(usize, char, char, Layout)> {
    let mut chars = text.char_indices().peekable();

    while let Some((index, c)) = chars.next() {
        let rest = &text[index..];
        if rest.starts_with("//") {
            skip_line(&mut chars);
            continue;
        }
        if let Some(hashes) = raw_string_start(text, index) {
            skip_raw_string(&mut chars, hashes);
            continue;
        }
        match c {
            '"' => {
                skip_string(&mut chars);
                continue;
            }
            '\'' => {
                skip_char_literal(&mut chars);
                continue;
            }
            _ => {}
        }

        let preceded_by_ident = text[..index]
            .chars()
            .next_back()
            .is_some_and(|c| c == '_' || c.is_alphanumeric());
        if preceded_by_ident {
            continue;
        }
        for (name, open, close, layout) in SUGAR_MACROS {
            // The printer may break between the name and its delimiter, so allow whitespace.
            if rest.starts_with(name) && rest[name.len()..].trim_start().starts_with(*open) {
                return Some((index, *open, *close, *layout));
            }
        }
    }

    None
}

/// Whether a raw string literal (`r"…"`, `r#"…"#`, or the byte form `br"…"`) starts at `index`,
/// and if so how many `#` it opens with.
pub fn raw_string_start(text: &str, index: usize) -> Option<usize> {
    let rest = &text[index..];
    let after_r = rest.strip_prefix("br").or_else(|| rest.strip_prefix('r'))?;
    let starts_a_token = !text[..index]
        .chars()
        .next_back()
        .is_some_and(|c| c == '_' || c.is_alphanumeric());
    let hashes = after_r.chars().take_while(|c| *c == '#').count();
    (starts_a_token && after_r[hashes..].starts_with('"')).then_some(hashes)
}

/// Consume a raw string literal from its prefix through the closing `"` and `hashes` `#`s.
pub fn skip_raw_string(chars: &mut Peekable<CharIndices<'_>>, hashes: usize) {
    // Through the opening quote.
    for (_, c) in chars.by_ref() {
        if c == '"' {
            break;
        }
    }
    while let Some((_, c)) = chars.next() {
        if c == '"' {
            let mut closing = 0;
            while closing < hashes && chars.next_if(|(_, c)| *c == '#').is_some() {
                closing += 1;
            }
            if closing == hashes {
                return;
            }
        }
    }
}

/// Consume a line comment through the end of its line.
pub fn skip_line(chars: &mut Peekable<CharIndices<'_>>) {
    for (_, c) in chars.by_ref() {
        if c == '\n' {
            return;
        }
    }
}

/// The offset of the delimiter that closes a body starting at `text`, or `None` when the
/// delimiters do not balance. Delimiters inside a string or character literal are skipped.
pub fn body_end(text: &str, open: char, close: char) -> Option<usize> {
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
pub fn skip_string(chars: &mut impl Iterator<Item = (usize, char)>) {
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
pub fn skip_char_literal(chars: &mut (impl Iterator<Item = (usize, char)> + Clone)) {
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
