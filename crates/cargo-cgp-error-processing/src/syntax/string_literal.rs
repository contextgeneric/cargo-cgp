//! Telling text inside a string literal from the code around it, so a delimiter or a path inside a
//! `Symbol!("…")` tag is never read as code.

/// The byte offset just past the string literal that opens at `open`, or the end of `text` when it
/// does not close. A backslash escapes the character after it, so `\"` does not close the literal.
///
/// Precondition: `text[open..].starts_with('"')`.
pub fn string_literal_end(text: &str, open: usize) -> usize {
    let contents = open + 1;
    let mut escaped = false;
    for (offset, c) in text[contents..].char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return contents + offset + 1;
        }
    }
    text.len()
}

/// The characters of `text` outside its string literals, with their byte offsets. The quotes
/// themselves count as inside.
pub fn code_chars(text: &str) -> impl Iterator<Item = (usize, char)> + '_ {
    let mut resume = 0;
    text.char_indices().filter(move |&(index, c)| {
        if index < resume {
            return false;
        }
        if c == '"' {
            resume = string_literal_end(text, index);
            return false;
        }
        true
    })
}
