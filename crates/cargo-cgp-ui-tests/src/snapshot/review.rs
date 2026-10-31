//! Comparing output with one snapshot, or rewriting it.

use std::fs;
use std::path::Path;

use crate::snapshot::diff::format_diff;
use crate::snapshot::outcome::Outcome;

/// Compare `actual` against the snapshot at `path`, or rewrite it when `bless` is set. `label`
/// names the pass in a mismatch report. A missing snapshot is a mismatch unless blessing, so a
/// deleted snapshot cannot pass as an empty one. Comparison ignores trailing whitespace so a
/// snapshot's single trailing newline never matters.
pub fn review(path: &Path, actual: &str, bless: bool, label: &str) -> Outcome {
    if bless {
        fs::write(path, ensure_trailing_newline(actual)).expect("writing snapshot");
        return Outcome::Blessed;
    }

    let Ok(expected) = fs::read_to_string(path) else {
        return Outcome::Mismatch(format!(
            "  --- {label}: missing snapshot {}; run with `--bless` to create it\n",
            path.display()
        ));
    };
    if expected.trim_end() == actual.trim_end() {
        Outcome::Ok
    } else {
        Outcome::Mismatch(format_diff(label, expected.trim_end(), actual.trim_end()))
    }
}

/// Guarantee exactly one trailing newline, so blessed snapshots end cleanly regardless
/// of whether the compiler output already ended in one.
pub fn ensure_trailing_newline(text: &str) -> String {
    format!("{}\n", text.trim_end())
}
