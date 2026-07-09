//! Tests that a non-CGP diagnostic passes through preprocessing unchanged.
//!
//! These exercise `process_cgp_errors` the way the design intends — from a *serialized*
//! fixture, with no compiler and no `cargo-cgp` process in the loop — which is what the
//! stateless signature buys. `tests/fixtures/sample_diagnostics.json` is a real rustc JSON
//! diagnostic stream (two plain-Rust errors and two failure-notes, no CGP constructs),
//! captured once and committed. Because it holds nothing CGP, the preprocessors are all
//! no-ops: the diagnostics come out untouched and `has_cgp_error` stays `false`. The
//! per-preprocessor transformations are tested in `preprocess.rs`.

use cargo_cgp_error_processing::cargo_metadata::diagnostic::Diagnostic;
use cargo_cgp_error_processing::process_cgp_errors;

/// Deserialize the committed fixture into the diagnostics the processor consumes.
fn sample_diagnostics() -> Vec<Diagnostic> {
    let json = include_str!("fixtures/sample_diagnostics.json");
    serde_json::from_str(json).expect("deserializing the sample diagnostics fixture")
}

#[test]
fn non_cgp_diagnostics_pass_through_unchanged() {
    let input = sample_diagnostics();
    assert_eq!(
        input.len(),
        4,
        "fixture should carry the four captured diagnostics"
    );

    let output = process_cgp_errors(input.clone());

    // Preprocessing is per-diagnostic, so the count is preserved. With no CGP constructs
    // in the input, every diagnostic is untouched and not flagged as CGP.
    assert_eq!(output.len(), input.len());
    for (processed, original) in output.iter().zip(&input) {
        assert_eq!(&processed.diagnostic, original);
        assert!(!processed.has_cgp_error);
    }
}

#[test]
fn rendered_matches_the_underlying_diagnostic() {
    let input = sample_diagnostics();
    let output = process_cgp_errors(input.clone());

    // The render stage prints `rendered` to reproduce rustc's own pretty output, so a
    // pass-through diagnostic must expose exactly what rustc rendered.
    for (processed, original) in output.iter().zip(&input) {
        assert_eq!(processed.rendered(), original.rendered.as_deref());
    }
}
