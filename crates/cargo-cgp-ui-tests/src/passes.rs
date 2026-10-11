//! The verification passes and checks each fixture goes through.
//!
//! Three passes run per fixture and compare against a committed snapshot. Two are about what the
//! compiler *says*, and the third is about what the macros *generate*:
//!
//! - [`cgp_stderr_pass`] compares the tool's stderr to `.cgp.stderr`, the end-to-end check that
//!   the whole binary produces the expected output.
//! - [`rust_stderr_pass`] compares plain `cargo check`'s stderr to `.rust.stderr`, the "before"
//!   the tool improves on.
//! - [`expand_pass`] runs `cargo cgp expand` and compares its stdout to `.expand.rs`, the code the
//!   compiler was actually given, which is where the answer to "why does it say that?" usually is.
//!
//! Two checks compare the passes with each other rather than with a snapshot:
//!
//! - [`status_check`]: the tool and plain `cargo check` must agree on whether the fixture
//!   compiles.
//! - [`pass_through_check`]: for a fixture under `pass-through/`, a plain Rust error the tool must
//!   not touch, the tool's output must equal plain rustc's, unless the fixture is marked
//!   `//@rewrites-falsely:` until its fix lands.
//!
//! [`header_check`] holds every error fixture to its structured header: the `//@quality:` it
//! declares, and the exact set of `[CGP-Exxx]` codes its `//@codes:` lists, checked against the
//! tool's output.
//!
//! A fixture marked `//@json` also runs [`json_pass`], which pins the diagnostics cargo's JSON
//! stream carries, the form an editor reads.

use std::path::Path;

use crate::directives::Directives;
use crate::harness::{self, Run};
use crate::normalize::{normalize, normalize_baseline, normalize_source, replace_paths};
use crate::snapshot::{Outcome, cgp_stderr_path, expand_path, json_path, review, rust_stderr_path};

/// The machine-specific paths every pass normalizes away.
pub struct Paths<'a> {
    /// The worker's throwaway crate.
    pub harness_crate: &'a Path,
    /// The sibling `cgp` checkout.
    pub cgp_root: &'a Path,
    /// The toolchain sysroot.
    pub sysroot: &'a Path,
}

/// Review the tool's stderr from `run` against `.cgp.stderr`.
pub fn cgp_stderr_pass(run: &Run, fixture: &Path, paths: &Paths, bless: bool) -> Outcome {
    let actual = normalize(
        &run.stderr,
        paths.harness_crate,
        paths.cgp_root,
        paths.sysroot,
    );
    review(&cgp_stderr_path(fixture), &actual, bless, "cgp")
}

/// Review plain `cargo check`'s stderr from `run` against `.rust.stderr`.
pub fn rust_stderr_pass(run: &Run, fixture: &Path, paths: &Paths, bless: bool) -> Outcome {
    let actual = normalize_baseline(
        &run.stderr,
        paths.harness_crate,
        paths.cgp_root,
        paths.sysroot,
    );
    review(&rust_stderr_path(fixture), &actual, bless, "rust")
}

/// Run `cargo cgp expand` on the fixture and review its expansion against `.expand.rs` — the Rust
/// its CGP macros generate, with CGP's type-level constructs resugared.
pub fn expand_pass(fixture: &Path, paths: &Paths, bless: bool) -> Outcome {
    let raw = harness::run_fixture_expand(paths.harness_crate, fixture);
    let actual = normalize_source(&raw, paths.harness_crate, paths.cgp_root, paths.sysroot);
    review(&expand_path(fixture), &actual, bless, "expand")
}

/// Run `cargo cgp check --message-format=json` on the fixture and review its compiler messages,
/// reduced by [`reduce_json`], against `.cgp.json`.
pub fn json_pass(fixture: &Path, paths: &Paths, bless: bool) -> Outcome {
    let stream = harness::run_fixture_json(paths.harness_crate, fixture);
    let reduced = reduce_json(&stream);
    let actual = replace_paths(&reduced, paths.harness_crate, paths.cgp_root, paths.sysroot);
    review(&json_path(fixture), &actual, bless, "json")
}

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

/// Check that a `pass-through/` fixture's tool output equals plain rustc's. Both are compared as
/// [`normalize_baseline`] leaves them, so the baseline's run-to-run spine truncation cannot cause a
/// difference. A fixture marked `//@rewrites-falsely:` is expected to differ; once it no longer
/// does, the marker is reported so its fix removes it.
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

/// The distinct `CGP-Exxx` codes in `text`, sorted.
pub fn cgp_codes(text: &str) -> Vec<String> {
    let mut codes: Vec<String> = text
        .match_indices("CGP-E")
        .filter_map(|(at, _)| {
            let code = text.get(at..at + 8)?;
            code[5..]
                .bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| code.to_owned())
        })
        .collect();
    codes.sort();
    codes.dedup();
    codes
}

/// Reduce cargo's JSON message stream to the compiler messages an editor reads, as pretty JSON:
/// for each message its level, code, text, spans (file, start line and column, primary flag,
/// label), and children (level and text). Cargo's artifact and build-finished lines, and the
/// `rendered` text (which the `.cgp.stderr` pass already pins), are left out.
pub fn reduce_json(stream: &str) -> String {
    use serde_json::{Value, json};

    let field = |value: &Value, key: &str| value.get(key).cloned().unwrap_or(Value::Null);
    let spans = |message: &Value| -> Vec<Value> {
        message
            .get("spans")
            .and_then(Value::as_array)
            .map(|spans| {
                spans
                    .iter()
                    .map(|span| {
                        json!({
                            "file": field(span, "file_name"),
                            "line": field(span, "line_start"),
                            "column": field(span, "column_start"),
                            "primary": field(span, "is_primary"),
                            "label": field(span, "label"),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let messages: Vec<Value> = stream
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|record| record.get("reason").and_then(Value::as_str) == Some("compiler-message"))
        .filter_map(|record| record.get("message").cloned())
        .map(|message| {
            let children: Vec<Value> = message
                .get("children")
                .and_then(Value::as_array)
                .map(|children| {
                    children
                        .iter()
                        .map(|child| {
                            json!({
                                "level": field(child, "level"),
                                "message": field(child, "message"),
                                "spans": spans(child),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            json!({
                "level": field(&message, "level"),
                "code": message.get("code").and_then(|code| code.get("code")).cloned().unwrap_or(Value::Null),
                "message": field(&message, "message"),
                "spans": spans(&message),
                "children": children,
            })
        })
        .collect();

    serde_json::to_string_pretty(&messages).expect("serializing the reduced JSON messages")
}
