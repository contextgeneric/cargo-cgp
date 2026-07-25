//! Comparing a fixture's output against its committed snapshot, or blessing it.

use std::fs;
use std::path::{Path, PathBuf};

/// The result of reviewing one fixture.
pub enum Outcome {
    /// Output matched the committed snapshot.
    Ok,
    /// Snapshot was (re)written from the current output.
    Blessed,
    /// Output differed from the committed snapshot; the carried string is the rendered
    /// diff, held rather than printed so a parallel run can emit it in fixture order.
    Mismatch(String),
}

/// The `.cgp.stderr` snapshot path beside a fixture (`foo.rs` → `foo.cgp.stderr`) — the
/// output `cargo-cgp` renders, the "after" the tool exists to produce.
pub fn cgp_stderr_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("cgp.stderr")
}

/// The `.rust.stderr` snapshot path beside a fixture (`foo.rs` → `foo.rust.stderr`) — the
/// output plain `cargo check` produces for the same fixture, recorded as the "before" so a
/// reader can see what `cargo-cgp` changes.
pub fn rust_stderr_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("rust.stderr")
}

/// The `.expand.rs` snapshot path beside a fixture (`foo.rs` → `foo.expand.rs`) — the Rust the
/// fixture's CGP macros generate, as `cargo cgp expand` shows it. It records what the macros
/// *produce*, where the two `.stderr` snapshots record what the compiler says about it.
pub fn expand_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("expand.rs")
}

/// Compare `actual` against the snapshot at `path`, or rewrite it when `bless` is set. On
/// a mismatch, [`Outcome::Mismatch`] carries the rendered diff (for the caller to print).
/// Comparison ignores trailing whitespace so a snapshot's single trailing newline never
/// matters.
pub fn review(path: &Path, actual: &str, bless: bool) -> Outcome {
    if bless {
        fs::write(path, ensure_trailing_newline(actual)).expect("writing snapshot");
        return Outcome::Blessed;
    }

    let expected = fs::read_to_string(path).unwrap_or_default();
    if expected.trim_end() == actual.trim_end() {
        Outcome::Ok
    } else {
        Outcome::Mismatch(format_diff(expected.trim_end(), actual.trim_end()))
    }
}

/// Guarantee exactly one trailing newline, so blessed snapshots end cleanly regardless
/// of whether the compiler output already ended in one.
fn ensure_trailing_newline(text: &str) -> String {
    format!("{}\n", text.trim_end())
}

/// Render the expected and actual output as two labeled blocks. Snapshots are small, so
/// full blocks are clearer than a computed line diff and never mislead.
fn format_diff(expected: &str, actual: &str) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    let _ = writeln!(out, "  --- expected (committed snapshot) ---");
    for line in expected.lines() {
        let _ = writeln!(out, "  | {line}");
    }
    let _ = writeln!(out, "  --- actual (cargo-cgp output) ---");
    for line in actual.lines() {
        let _ = writeln!(out, "  | {line}");
    }
    let _ = writeln!(out, "  ---");
    out
}
