//! The checks that compare a fixture's passes with each other or with its header.

use crate::directives::Directives;
use crate::passes::codes::cgp_codes;
use crate::snapshot::Outcome;

/// Check that the tool and plain `cargo check` agree on whether the fixture compiles. A crash
/// with no output, or the tool accepting what rustc rejects, would otherwise pass whenever the
/// snapshot happened to match.
pub fn status_check(plain_ok: bool, tool_ok: bool) -> Outcome {
    if plain_ok == tool_ok {
        return Outcome::Ok;
    }
    let describe = |ok: bool| if ok { "compiles" } else { "fails" };
    Outcome::Mismatch(format!(
        "  --- status: plain `cargo check` {} but `cargo cgp check` {} ---\n",
        describe(plain_ok),
        describe(tool_ok)
    ))
}

/// Check that `cargo cgp expand` fails exactly when it reported an error. Expansion stops before
/// type-checking, so its status differs from `cargo check`'s, and this ties it to its own stderr: a
/// compile error must not end in success, and a clean expansion must not end in failure. `stderr` is
/// the command's as run with `-q`, so a line starting `error` is a real report.
///
/// Property: for all (`success`, `stderr`), the result is `Ok` iff `success` differs from whether
/// some line of `stderr` starts with `error`.
pub fn expand_status_check(success: bool, stderr: &str) -> Outcome {
    let reported_error = stderr.lines().any(|line| line.starts_with("error"));
    if success != reported_error {
        return Outcome::Ok;
    }
    Outcome::Mismatch(if success {
        "  --- expand status: `cargo cgp expand` reported an error but exited 0 ---\n".to_owned()
    } else {
        "  --- expand status: `cargo cgp expand` failed without reporting an error ---\n".to_owned()
    })
}

/// Check that a `pass-through/` fixture's tool output equals plain rustc's. Both are compared as
/// [`normalize_baseline`](crate::normalize::normalize_baseline) leaves them, so the baseline's run-to-run spine truncation cannot cause a
/// difference. A fixture marked `//@rewrites-falsely:` is expected to differ; once it no longer
/// does, the marker is reported so the fix removes it.
pub fn pass_through_check(baseline: &str, tool: &str, rewrites_falsely: Option<&str>) -> Outcome {
    let equal = baseline.trim_end() == tool.trim_end();
    match (equal, rewrites_falsely) {
        (true, None) | (false, Some(_)) => Outcome::Ok,
        (false, None) => Outcome::Mismatch(crate::snapshot::format_diff(
            "pass-through: `-` plain rustc, `+` cargo-cgp (they must be equal)",
            baseline.trim_end(),
            tool.trim_end(),
        )),
        (true, Some(reason)) => Outcome::Mismatch(format!(
            "  --- pass-through: the tool's output now equals plain rustc's; remove the \
             `//@rewrites-falsely: {reason}` marker ---\n"
        )),
    }
}

/// Check an error fixture's structured header against the tool's output: it must declare its
/// `//@quality:`, and its `//@codes:` must list exactly the `[CGP-Exxx]` codes in `tool_output`.
/// Fixtures under `ok/`, `pass-through/`, and `robustness/` are `exempt`: a clean compile carries no
/// codes, a pass-through fixture is held to plain rustc's output instead, and a robustness fixture
/// pins how the tool degrades when it fails, which no quality describes.
pub fn header_check(exempt: bool, directives: &Directives, tool_output: &str) -> Outcome {
    if exempt {
        return Outcome::Ok;
    }
    let mut problems = Vec::new();
    if directives.quality.is_none() {
        problems.push("no `//@quality:` directive".to_owned());
    }
    match &directives.codes {
        None => problems.push("no `//@codes:` directive".to_owned()),
        Some(declared) => {
            let actual = cgp_codes(tool_output);
            let mut declared = declared.clone();
            declared.sort();
            declared.dedup();
            if declared != actual {
                let list = |codes: &[String]| {
                    if codes.is_empty() {
                        "none".to_owned()
                    } else {
                        codes.join(" ")
                    }
                };
                problems.push(format!(
                    "`//@codes: {}` but the output carries {}",
                    list(&declared),
                    list(&actual)
                ));
            }
        }
    }
    if problems.is_empty() {
        Outcome::Ok
    } else {
        Outcome::Mismatch(format!("  --- header: {} ---\n", problems.join("; ")))
    }
}
