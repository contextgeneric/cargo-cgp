//! Preprocessor: resugar the `Symbol!` type-level string.
//!
//! `Symbol!("xy")` expands to `Symbol<2, Chars<'x', Chars<'y', Nil>>>` — a length and a
//! right-folded character list terminated by `Nil` (see the CGP `Symbol!` reference). This
//! preprocessor reverses that spine back to `Symbol!("xy")` in the diagnostic text.
//!
//! Resugaring is applied only on an *exact* structural match: the length must equal the
//! decoded string's byte length, the spine must be `Chars`/`Nil` all the way down, and
//! each `Chars` head must be a single plain character literal. Anything else is left
//! untouched, because a differently-shaped type that merely shares the `Symbol` or `Chars`
//! name must not be silently rewritten. This caution applies to every resugaring
//! preprocessor, not just this one.

use crate::diagnostic::CgpDiagnostic;
use crate::preprocess::text::map_diagnostic_text;

/// Rewrite every well-formed `Symbol<…>` spine in the diagnostic text to its `Symbol!("…")`
/// surface form, and set `has_cgp_error` if any was rewritten — a resugared `Symbol!` is a
/// CGP construct, so recognizing one flags the diagnostic as CGP-related. Runs after
/// [`strip_cgp_prefixes`](super::strip_cgp_prefixes), so it matches the bare `Symbol`,
/// `Chars`, and `Nil` names left once the CGP path prefixes are gone.
pub fn resugar_symbol(mut diagnostic: CgpDiagnostic) -> CgpDiagnostic {
    if map_diagnostic_text(&mut diagnostic.diagnostic, resugar_symbols_in_text) {
        diagnostic.has_cgp_error = true;
    }
    diagnostic
}

/// Replace each exact `Symbol<…>` spine in `text`, returning the result and whether any
/// replacement happened. A `Symbol<` that does not parse as an exact spine is emitted
/// unchanged and scanning resumes after it.
fn resugar_symbols_in_text(text: &str) -> (String, bool) {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut changed = false;

    while let Some(index) = rest.find("Symbol<") {
        out.push_str(&rest[..index]);
        let candidate = &rest[index..];
        match parse_symbol(candidate) {
            Some((name, consumed)) => {
                out.push_str(&format!("Symbol!({name:?})"));
                rest = &candidate[consumed..];
                changed = true;
            }
            None => {
                out.push_str("Symbol<");
                rest = &candidate["Symbol<".len()..];
            }
        }
    }

    out.push_str(rest);
    (out, changed)
}

/// Parse a `Symbol<LEN, spine>` at the start of `input` (which begins with `Symbol<`).
/// Returns the decoded string and the number of bytes consumed, or `None` on any
/// mismatch. `LEN` is the string's byte length (see the `Symbol!` reference), so it is
/// checked against `name.len()`, not the character count.
fn parse_symbol(input: &str) -> Option<(String, usize)> {
    let rest = input.strip_prefix("Symbol<")?.trim_start();
    let (declared_len, rest) = parse_usize(rest)?;
    let rest = rest.trim_start().strip_prefix(',')?.trim_start();
    let (name, rest) = parse_chars_spine(rest)?;
    let rest = rest.trim_start().strip_prefix('>')?;

    if declared_len != name.len() {
        return None;
    }

    Some((name, input.len() - rest.len()))
}

/// Parse a run of ASCII digits into a `usize`, returning it and the remaining text.
fn parse_usize(input: &str) -> Option<(usize, &str)> {
    let end = input
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(input.len());
    if end == 0 {
        return None;
    }
    Some((input[..end].parse().ok()?, &input[end..]))
}

/// Parse a `Chars<'c', tail>` chain terminated by `Nil`, decoding it to a string and
/// returning the remaining text after the chain.
fn parse_chars_spine(input: &str) -> Option<(String, &str)> {
    if let Some(rest) = input.strip_prefix("Nil") {
        return Some((String::new(), rest));
    }

    let rest = input.strip_prefix("Chars<")?.trim_start();
    let (head, rest) = parse_char_literal(rest)?;
    let rest = rest.trim_start().strip_prefix(',')?.trim_start();
    let (tail, rest) = parse_chars_spine(rest)?;
    let rest = rest.trim_start().strip_prefix('>')?;

    let mut name = String::new();
    name.push(head);
    name.push_str(&tail);
    Some((name, rest))
}

/// Parse a single plain character literal `'c'`, returning the character and the remaining
/// text. An escaped or multi-character literal returns `None`, so the enclosing symbol is
/// left unsugared rather than decoded by guesswork.
fn parse_char_literal(input: &str) -> Option<(char, &str)> {
    let rest = input.strip_prefix('\'')?;
    let head = rest.chars().next()?;
    if head == '\'' || head == '\\' {
        return None;
    }
    let rest = rest.strip_prefix(head)?.strip_prefix('\'')?;
    Some((head, rest))
}
