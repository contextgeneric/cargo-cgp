//! The line diff a snapshot mismatch reports.

/// How many unchanged lines to show around each change in a diff.
pub const CONTEXT: usize = 2;

/// Render a line diff of the committed snapshot against the actual output, headed by the pass
/// `label`: `-` marks a line only in the snapshot, `+` a line only in the output, and up to
/// `CONTEXT` unchanged lines surround each change.
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
pub enum Line<'a> {
    /// A line both sides hold.
    Same(&'a str),
    /// A line only the expected side holds.
    Removed(&'a str),
    /// A line only the actual side holds.
    Added(&'a str),
}

/// The line-level diff of `old` against `new`, from their longest common subsequence.
/// Snapshots hold at most a few hundred lines, so the quadratic table is cheap.
pub fn diff_lines<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<Line<'a>> {
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
