//! The three verification passes each fixture goes through.
//!
//! All three must agree on the rendered diagnostics, so they cross-check each other:
//!
//! - [`stderr_pass`] runs `cargo-cgp` directly and compares its stderr to `.stderr`.
//! - [`json_pass`] captures the raw JSON the tool sees, extracts the diagnostics, and
//!   compares them to `.output.json`.
//! - [`process_pass`] parses `.output.json`, runs it through `process_cgp_errors`, renders
//!   the result, and compares to `.stderr` — the pure unit pass, needing no compilation.
//!
//! The stderr and process passes share a target (`.stderr`) because rendering the
//! processed diagnostics must reproduce what the tool itself prints. They reuse the tool's
//! own capture (`parse_cargo_output`) and render (`emit_rendered`) code so the unit pass
//! cannot drift from the binary.

use std::fs;
use std::path::Path;

use cargo_cgp::check::{emit_rendered, parse_cargo_output};
use cargo_cgp_error_processing::process_cgp_errors;
use cargo_metadata::diagnostic::Diagnostic;

use crate::harness;
use crate::normalize::{normalize, normalize_json};
use crate::snapshot::{Outcome, output_json_path, review, stderr_path};

/// Pass 1 — run `cargo-cgp` on the fixture and review its stderr against `.stderr`.
pub fn stderr_pass(harness_crate: &Path, fixture: &Path, cgp_root: &Path, bless: bool) -> Outcome {
    let raw = harness::run_fixture(harness_crate, fixture);
    let actual = normalize(&raw, harness_crate, cgp_root);
    review(&stderr_path(fixture), &actual, bless)
}

/// Pass 2 — capture the raw JSON the tool sees, extract the diagnostics it feeds to
/// processing, and review them against `.output.json`.
pub fn json_pass(harness_crate: &Path, fixture: &Path, cgp_root: &Path, bless: bool) -> Outcome {
    let stdout = harness::run_fixture_json(harness_crate, fixture);
    let diagnostics = parse_cargo_output(&stdout).diagnostics;
    let json = serde_json::to_string_pretty(&diagnostics).expect("serializing the diagnostics");
    let actual = normalize_json(&json, harness_crate, cgp_root);
    review(&output_json_path(fixture), &actual, bless)
}

/// Pass 3 — the unit pass: parse `.output.json`, run `process_cgp_errors`, render the
/// result, and review it against `.stderr`. Needs no compilation, so it is the fast pass
/// for iterating on the processing implementation. Returns [`Outcome::Mismatch`] (with a
/// message) if the fixture has no committed `.output.json` to read yet.
pub fn process_pass(harness_crate: &Path, fixture: &Path, cgp_root: &Path, bless: bool) -> Outcome {
    let json_path = output_json_path(fixture);
    let json = match fs::read_to_string(&json_path) {
        Ok(json) => json,
        Err(_) => {
            return Outcome::Mismatch(format!(
                "  missing {} — run the full suite with --bless first\n",
                json_path.display()
            ));
        }
    };

    let diagnostics: Vec<Diagnostic> = match serde_json::from_str(&json) {
        Ok(diagnostics) => diagnostics,
        Err(error) => {
            return Outcome::Mismatch(format!(
                "  {} is not valid diagnostics JSON: {error}\n",
                json_path.display()
            ));
        }
    };

    let rendered = render_from_json(diagnostics);
    let actual = normalize(&rendered, harness_crate, cgp_root);
    review(&stderr_path(fixture), &actual, bless)
}

/// Render the diagnostics committed in `.output.json` through `process_cgp_errors`,
/// without normalizing or comparing — used by `--print` to show the process pass's raw
/// output. Returns an error message string if the fixture has no readable JSON.
pub fn print_process_output(fixture: &Path) -> String {
    let json_path = output_json_path(fixture);
    match fs::read_to_string(&json_path).ok().and_then(|json| {
        serde_json::from_str::<Vec<Diagnostic>>(&json)
            .ok()
            .map(render_from_json)
    }) {
        Some(rendered) => rendered,
        None => format!("(no readable {})\n", json_path.display()),
    }
}

/// Process the diagnostics and render them to a string, reusing the tool's own renderer.
fn render_from_json(diagnostics: Vec<Diagnostic>) -> String {
    let processed = process_cgp_errors(diagnostics);
    let mut buffer = Vec::new();
    emit_rendered(&mut buffer, &processed).expect("rendering processed diagnostics");
    String::from_utf8(buffer).expect("rendered diagnostics are UTF-8")
}
