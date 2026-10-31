//! The two layouts of a macro body: the printer's line breaks kept, or joined onto one line.

/// How a macro body is laid out after tightening.
#[derive(Clone, Copy)]
pub enum Layout {
    /// Keep the printer's line breaks, so a long list stays broken across lines.
    AsPrinted,
    /// Join the body onto the macro's line, between single spaces: `Struct! { width: f64 }`.
    Inline,
}

/// A brace body with the printer's line breaks and their indentation collapsed to single spaces,
/// and the space it leaves at either end trimmed. String literals are copied whole.
pub fn join_lines(body: &str) -> String {
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
pub fn tighten(body: &str) -> String {
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
pub fn copy_string(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, out: &mut String) {
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
