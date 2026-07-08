//! Normalizing tool output into a portable snapshot.
//!
//! Running each fixture through the next-generation trait solver produces the richer
//! diagnostics cargo-cgp exists to surface, but those diagnostics carry a couple of
//! machine-specific details that must not be committed: the absolute path of the sibling
//! `cgp` checkout (in cross-crate notes) and of the throwaway crate, plus a note pointing
//! at a hash-named temp file when a long type is elided. This module rewrites the paths to
//! stable placeholders and drops the temp-file note, so a snapshot depends only on the
//! diagnostic content.

use std::path::Path;

/// Note lines that name a volatile, hash-bearing temp file. They carry no diagnostic
/// content, so they are dropped rather than pattern-normalized.
const DROP_MARKERS: &[&str] = &[
    "the full name for the type has been written to",
    "consider using `--verbose` to print the full type name",
];

/// Rewrite `raw` tool output into its committed-snapshot form: drop the temp-file notes,
/// and replace the throwaway-crate and cgp-checkout absolute paths with `$DIR` and `$CGP`.
pub fn normalize(raw: &str, harness_dir: &Path, cgp_root: &Path) -> String {
    let dir = harness_dir.display().to_string();
    let cgp = cgp_root.display().to_string();

    raw.lines()
        .filter(|line| !DROP_MARKERS.iter().any(|marker| line.contains(marker)))
        .map(|line| line.replace(&dir, "$DIR").replace(&cgp, "$CGP"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn replaces_paths_and_drops_temp_note() {
        let raw = "\
error: boom
  --> /home/x/cgp/crates/core/cgp-field/src/traits/has_field.rs:50:1
   = note: the full name for the type has been written to '/home/x/proj/target/ui-harness/target/debug/deps/ui-abc.long-type-9.txt'
   = note: consider using `--verbose` to print the full type name to the console
   = help: keep this";
        let out = normalize(
            raw,
            &PathBuf::from("/home/x/proj/target/ui-harness"),
            &PathBuf::from("/home/x/cgp"),
        );
        assert_eq!(
            out,
            "\
error: boom
  --> $CGP/crates/core/cgp-field/src/traits/has_field.rs:50:1
   = help: keep this"
        );
    }
}
