//! The form of a `CGP-Exxx` code, for the two readers of one: a fixture's `//@codes:` and the
//! tool's output.

/// The `CGP-Exxx` code `text` starts with: `CGP-E` followed by three digits.
pub fn cgp_code_at(text: &str) -> Option<&str> {
    let digits = text.strip_prefix("CGP-E")?.get(..3)?;
    digits
        .bytes()
        .all(|b| b.is_ascii_digit())
        .then(|| &text[.."CGP-E".len() + digits.len()])
}
