//! Comparing a fixture's output against its committed snapshot, or blessing it.

use std::fs;
use std::path::{Path, PathBuf};

/// The result of reviewing one fixture.
pub enum Outcome {
    /// Output matched the committed snapshot.
    Ok,
    /// Snapshot was (re)written from the current output.
    Blessed,
    /// Output differed from the committed snapshot, or a check across passes failed; the
    /// carried string is the rendered report, held so the fixture's block prints whole.
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

/// The `.cgp.json` snapshot path beside a fixture (`foo.rs` → `foo.cgp.json`) — the tool's
/// diagnostics as cargo's JSON stream carries them, for the fixtures that ask for the JSON pass.
pub fn json_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("cgp.json")
}

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
fn ensure_trailing_newline(text: &str) -> String {
    format!("{}\n", text.trim_end())
}

/// How many unchanged lines to show around each change in a diff.
const CONTEXT: usize = 2;

/// Render a line diff of the committed snapshot against the actual output, headed by the pass
/// `label`: `-` marks a line only in the snapshot, `+` a line only in the output, and up to
/// [`CONTEXT`] unchanged lines surround each change.
pub fn format_diff(label: &str, expected: &str, actual: &str) -> String {
    use std::fmt::Write;

    let old: Vec<&str> = expected.lines().collect();
    let new: Vec<&str> = actual.lines().collect();
    let ops = diff_lines(&old, &new);

    // Keep each line within CONTEXT lines of a change.
    let changed: Vec<bool> = ops.iter().map(|op| !matches!(op, Line::Same(_))).collect();
    let keep: Vec<bool> = (0..ops.len())
        .map(|i| {
            let lo = i.saturating_sub(CONTEXT);
            let hi = (i + CONTEXT + 1).min(ops.len());
            changed[lo..hi].iter().any(|&c| c)
        })
        .collect();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "  --- {label}: `-` committed snapshot, `+` actual output ---"
    );
    let mut skipped = false;
    for (op, keep) in ops.iter().zip(keep) {
        if !keep {
            skipped = true;
            continue;
        }
        if skipped {
            let _ = writeln!(out, "  ...");
            skipped = false;
        }
        let _ = match op {
            Line::Same(line) => writeln!(out, "    {line}"),
            Line::Removed(line) => writeln!(out, "  - {line}"),
            Line::Added(line) => writeln!(out, "  + {line}"),
        };
    }
    if skipped {
        let _ = writeln!(out, "  ...");
    }
    out
}

/// One line of a diff.
enum Line<'a> {
    Same(&'a str),
    Removed(&'a str),
    Added(&'a str),
}

/// The line-level diff of `old` against `new`, from their longest common subsequence.
/// Snapshots hold at most a few hundred lines, so the quadratic table is cheap.
fn diff_lines<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<Line<'a>> {
    let (n, m) = (old.len(), new.len());
    // lcs[i][j]: the LCS length of old[i..] and new[j..].
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if old[i] == new[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }

    let mut ops = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if old[i] == new[j] {
            ops.push(Line::Same(old[i]));
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            ops.push(Line::Removed(old[i]));
            i += 1;
        } else {
            ops.push(Line::Added(new[j]));
            j += 1;
        }
    }
    ops.extend(old[i..].iter().map(|line| Line::Removed(line)));
    ops.extend(new[j..].iter().map(|line| Line::Added(line)));
    ops
}
