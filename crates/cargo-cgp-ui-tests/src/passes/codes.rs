//! Finding the `CGP-Exxx` codes in a tool's output.

/// The distinct `CGP-Exxx` codes in `text`, sorted.
pub fn cgp_codes(text: &str) -> Vec<String> {
    let mut codes: Vec<String> = text
        .match_indices("CGP-E")
        .filter_map(|(at, _)| {
            let code = text.get(at..at + 8)?;
            code[5..]
                .bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| code.to_owned())
        })
        .collect();
    codes.sort();
    codes.dedup();
    codes
}
