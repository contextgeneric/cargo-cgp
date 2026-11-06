//! Normalizing tool output into a portable snapshot.
//!
//! Running each fixture through the next-generation trait solver produces the richer
//! diagnostics cargo-cgp exists to surface, but those diagnostics carry a few
//! machine-specific details that must not be committed: the absolute path of the sibling
//! `cgp` checkout (in cross-crate notes), of the throwaway crate, and of the toolchain
//! sysroot (in a note pointing into the standard library, which carries the user's home
//! directory, the pinned nightly's name, and the host target triple), plus a note pointing
//! at a hash-named temp file when a long type is elided. This module rewrites the paths to
//! stable placeholders and drops the temp-file note, so a snapshot depends only on the
//! diagnostic content. It applies to the rendered `.stderr` of both the tool pass and the
//! `cargo check` baseline pass.
//!
//! One further source of noise is *non-deterministic* rather than machine-specific, and confined
//! to the baseline. Plain `cargo check` renders a struct's "other `HasField` impls" suggestion by
//! spelling each field name as a `Symbol<N, Chars<'a', Chars<'b', …>>>` type-level spine, and
//! rustc truncates such a spine to `_` at a depth that varies between runs of the *same* compiler
//! (the diagnostic's "other impls" list uses a trimmed, length-budgeted printer whose cut point
//! shifts with the non-deterministic order the impls are listed in). [`normalize_baseline`]
//! therefore rewrites every such spine to a stable `Chars<..>` placeholder. The tool's output is
//! normalized without that collapse: the tool resugars a field name into `Symbol!("…")`, so a raw
//! `Chars<…>` spine in its output means the resugaring declined, which a snapshot must show.
//!
//! A compiler crash report (from the `robustness/` fixture that makes rustc panic on purpose) names
//! the panicking thread by its operating-system id, which differs on every run, so [`normalize`]
//! replaces it with `$TID`.

use std::path::Path;

/// Lines dropped from the rendered `.stderr` because they carry no diagnostic content.
/// The temp-file notes name a volatile, hash-bearing path; the `could not compile` line
/// is cargo's own build-failure summary, which is not part of a diagnostic, so dropping it
/// keeps the tool pass and the baseline pass comparable.
const DROP_MARKERS: &[&str] = &[
    "the full name for the type has been written to",
    "consider using `--verbose` to print the full type name",
    "error: could not compile ",
    // Cargo's own failure line for the compiler process. It appears only under coverage
    // instrumentation (`cargo llvm-cov`), which makes rustc exit through a path cargo reports.
    "error: process didn't exit successfully:",
];

/// Rewrite the tool's rendered `.stderr` output into its committed-snapshot form: drop the
/// content-free lines above, replace the throwaway-crate, cgp-checkout, and sysroot absolute paths
/// with `$DIR`, `$CGP`, and `$SYSROOT`, and a crash report's thread id with `$TID`.
pub fn normalize(raw: &str, harness_dir: &Path, cgp_root: &Path, sysroot: &Path) -> String {
    raw.lines()
        .filter(|line| !DROP_MARKERS.iter().any(|marker| line.contains(marker)))
        .map(|line| replace_thread_id(&replace_paths(line, harness_dir, cgp_root, sysroot)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Replace the thread id in a panic's first line, `thread 'rustc' (1234) panicked at …`, with
/// `$TID`; any other line is returned unchanged.
fn replace_thread_id(line: &str) -> String {
    let Some(rest) = line.strip_prefix("thread '") else {
        return line.to_owned();
    };
    let Some(name_end) = rest.find("' (") else {
        return line.to_owned();
    };
    let after_name = &rest[name_end + "' (".len()..];
    let digits = after_name.bytes().take_while(u8::is_ascii_digit).count();
    let tail = &after_name[digits..];
    if digits == 0 || !tail.starts_with(") panicked") {
        return line.to_owned();
    }
    format!("thread '{}' ($TID{tail}", &rest[..name_end])
}

/// Rewrite plain `cargo check`'s rendered `.stderr` into its committed-snapshot form: what
/// [`normalize`] does, plus the collapse of every `Chars<…>` spine, whose truncation in the
/// baseline varies between runs.
pub fn normalize_baseline(
    raw: &str,
    harness_dir: &Path,
    cgp_root: &Path,
    sysroot: &Path,
) -> String {
    normalize(raw, harness_dir, cgp_root, sysroot)
        .lines()
        .map(collapse_chars_spines)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Rewrite every `Chars<…>` type-level spine in `text` to a stable `Chars<..>` placeholder,
/// collapsing the whole balanced spine (`Chars<'m', Chars<'a', …, _>>` or `… Nil>`) at once. This
/// erases the run-to-run drift in how the baseline renders a `Symbol` field-name spine: the depth
/// at which rustc truncates it, and whether the terminal reads `_` or `Nil`, while leaving the
/// enclosing `Symbol<N, …>` (and its stable length `N`) intact. Scanning from each `Chars<` and
/// consuming to its matching `>` by angle-bracket depth means a nested spine is swallowed by its
/// outermost `Chars<`, so one placeholder replaces the entire spine.
fn collapse_chars_spines(text: &str) -> String {
    const OPEN: &str = "Chars<";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(OPEN) {
        out.push_str(&rest[..start]);
        out.push_str("Chars<..>");
        // Skip past the matched `Chars<` and everything up to its balancing `>`.
        let after_open = &rest[start + OPEN.len()..];
        let mut depth = 1i32;
        let mut end = after_open.len();
        for (i, ch) in after_open.char_indices() {
            match ch {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &after_open[end..];
    }
    out.push_str(rest);
    out
}

/// Rewrite an *expansion* into its committed-snapshot form.
///
/// Only the absolute paths are replaced. None of the diagnostic normalization applies here, and one
/// piece of it would actively hide a defect: `collapse_chars_spines` exists to absorb how rustc
/// truncates a `Chars` spine in a *diagnostic*, but in an expansion a raw `Chars<…>` spine means
/// the resugaring declined: exactly what the snapshot is there to show.
pub fn normalize_source(raw: &str, harness_dir: &Path, cgp_root: &Path, sysroot: &Path) -> String {
    raw.lines()
        .map(|line| replace_paths(line, harness_dir, cgp_root, sysroot))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Replace the throwaway-crate, cgp-checkout, and sysroot absolute paths with
/// `$DIR`/`$CGP`/`$SYSROOT`.
pub fn replace_paths(text: &str, harness_dir: &Path, cgp_root: &Path, sysroot: &Path) -> String {
    let dir = harness_dir.display().to_string();
    let cgp = cgp_root.display().to_string();
    let sysroot = sysroot.display().to_string();
    text.replace(&dir, "$DIR")
        .replace(&cgp, "$CGP")
        .replace(&sysroot, "$SYSROOT")
}
