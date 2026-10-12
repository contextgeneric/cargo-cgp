//! Bounding the list of implementors the driver's `--verbose` flag un-elides.
//!
//! When a trait bound fails, rustc's help lists the types that do implement the trait, eight at
//! most followed by "and N others", unless the session is verbose, when it lists every one. The
//! driver injects `--verbose` for the type names it un-elides, so a CGP crate's failure on a trait
//! as widely implemented as `IsProviderFor` would list hundreds of implementors. Cutting the list
//! back to rustc's own length restores exactly what plain `cargo check` prints.

/// How many candidates rustc names before it counts the rest, outside a verbose session.
pub const RUSTC_CANDIDATE_LIMIT: usize = 8;

/// `text` with its candidate list cut back to the length rustc prints outside a verbose session:
/// the first [`RUSTC_CANDIDATE_LIMIT`] candidates and "and N others". `None` when `text` is not
/// rustc's candidate-list help, or when rustc would print the whole list anyway (nine or fewer).
pub fn bound_candidate_list(text: &str) -> Option<String> {
    let (heading, list) = text.split_once('\n')?;
    if !is_candidate_heading(heading) {
        return None;
    }
    let candidates: Vec<&str> = list.split('\n').collect();
    if candidates.len() <= RUSTC_CANDIDATE_LIMIT + 1
        || !candidates
            .iter()
            .all(|candidate| candidate.starts_with("  "))
    {
        return None;
    }
    let mut bounded = heading.to_owned();
    for candidate in &candidates[..RUSTC_CANDIDATE_LIMIT] {
        bounded.push('\n');
        bounded.push_str(candidate);
    }
    bounded.push_str(&format!(
        "\nand {} others",
        candidates.len() - RUSTC_CANDIDATE_LIMIT
    ));
    Some(bounded)
}

/// Whether `line` opens rustc's candidate-list help: "the following (other) types implement trait
/// `T`:", or "`S` implements trait `T`:" when every candidate shares one type.
fn is_candidate_heading(line: &str) -> bool {
    line.ends_with("`:")
        && (line.starts_with("the following types implement trait `")
            || line.starts_with("the following other types implement trait `")
            || (line.starts_with('`') && line.contains("` implements trait `")))
}
