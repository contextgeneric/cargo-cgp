//! Comparing a fixture's output against its committed snapshot, or blessing it.

use std::fs;
use std::path::{Path, PathBuf};

/// The result of reviewing one fixture.
pub enum Outcome {
    /// Output matched the committed snapshot.
    Ok,
    /// Snapshot was (re)written from the current output.
    Blessed,
    /// Output differed from the committed snapshot; a diff was printed.
    Mismatch,
}

/// The `.stderr` snapshot path beside a fixture (`foo.rs` → `foo.stderr`).
pub fn stderr_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("stderr")
}

/// The `.output.json` snapshot path beside a fixture (`foo.rs` → `foo.output.json`) —
/// the diagnostics the tool captured and fed to `process_cgp_errors`.
pub fn output_json_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("output.json")
}

/// Compare `actual` against the snapshot at `path`, or rewrite it when `bless` is set. On
/// a mismatch, a diff is printed and [`Outcome::Mismatch`] returned. Comparison ignores
/// trailing whitespace so a snapshot's single trailing newline never matters.
pub fn review(path: &Path, actual: &str, bless: bool) -> Outcome {
    if bless {
        fs::write(path, ensure_trailing_newline(actual)).expect("writing snapshot");
        return Outcome::Blessed;
    }

    let expected = fs::read_to_string(path).unwrap_or_default();
    if expected.trim_end() == actual.trim_end() {
        Outcome::Ok
    } else {
        print_diff(expected.trim_end(), actual.trim_end());
        Outcome::Mismatch
    }
}

/// Guarantee exactly one trailing newline, so blessed snapshots end cleanly regardless
/// of whether the compiler output already ended in one.
fn ensure_trailing_newline(text: &str) -> String {
    format!("{}\n", text.trim_end())
}

/// Print the expected and actual output as two labeled blocks. Snapshots are small, so
/// full blocks are clearer than a computed line diff and never mislead.
fn print_diff(expected: &str, actual: &str) {
    eprintln!("  --- expected (committed .stderr) ---");
    for line in expected.lines() {
        eprintln!("  | {line}");
    }
    eprintln!("  --- actual (cargo-cgp output) ---");
    for line in actual.lines() {
        eprintln!("  | {line}");
    }
    eprintln!("  ---");
}
