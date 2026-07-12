//! Normalizing tool output into a portable snapshot.
//!
//! Running each fixture through the next-generation trait solver produces the richer
//! diagnostics cargo-cgp exists to surface, but those diagnostics carry a couple of
//! machine-specific details that must not be committed: the absolute path of the sibling
//! `cgp` checkout (in cross-crate notes) and of the throwaway crate, plus a note pointing
//! at a hash-named temp file when a long type is elided. This module rewrites the paths to
//! stable placeholders and drops the temp-file note, so a snapshot depends only on the
//! diagnostic content. It applies to the rendered `.stderr` of both the tool pass and the
//! plain-`cargo check` baseline pass.

use std::path::Path;

/// Lines dropped from the rendered `.stderr` because they carry no diagnostic content.
/// The temp-file notes name a volatile, hash-bearing path; the `could not compile` line
/// is cargo's own build-failure summary, which is not part of a diagnostic — so dropping it
/// keeps the tool pass and the baseline pass comparable.
const DROP_MARKERS: &[&str] = &[
    "the full name for the type has been written to",
    "consider using `--verbose` to print the full type name",
    "error: could not compile ",
];

/// Rewrite rendered `.stderr` output into its committed-snapshot form: drop the
/// content-free lines above, and replace the throwaway-crate and cgp-checkout absolute
/// paths with `$DIR` and `$CGP`.
pub fn normalize(raw: &str, harness_dir: &Path, cgp_root: &Path) -> String {
    raw.lines()
        .filter(|line| !DROP_MARKERS.iter().any(|marker| line.contains(marker)))
        .map(|line| replace_paths(line, harness_dir, cgp_root))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Replace the throwaway-crate and cgp-checkout absolute paths with `$DIR`/`$CGP`.
fn replace_paths(text: &str, harness_dir: &Path, cgp_root: &Path) -> String {
    let dir = harness_dir.display().to_string();
    let cgp = cgp_root.display().to_string();
    text.replace(&dir, "$DIR").replace(&cgp, "$CGP")
}
