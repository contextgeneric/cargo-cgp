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
pub fn snapshot_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("stderr")
}

/// Compare `actual` against the fixture's snapshot, or rewrite the snapshot when
/// `bless` is set. On a mismatch, a diff is printed and [`Outcome::Mismatch`] returned.
pub fn review(fixture: &Path, actual: &str, bless: bool) -> Outcome {
    let path = snapshot_path(fixture);

    if bless {
        fs::write(&path, ensure_trailing_newline(actual)).expect("writing snapshot");
        return Outcome::Blessed;
    }

    let expected = fs::read_to_string(&path).unwrap_or_default();
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
