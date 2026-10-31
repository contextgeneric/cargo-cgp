//! Finding the `CGP-Exxx` codes in a tool's output.

use crate::cgp_code::cgp_code_at;

/// The distinct `CGP-Exxx` codes in `text`, sorted.
pub fn cgp_codes(text: &str) -> Vec<String> {
    let mut codes: Vec<String> = text
        .match_indices("CGP-E")
        .filter_map(|(at, _)| cgp_code_at(&text[at..]))
        .map(str::to_owned)
        .collect();
    codes.sort();
    codes.dedup();
    codes
}
