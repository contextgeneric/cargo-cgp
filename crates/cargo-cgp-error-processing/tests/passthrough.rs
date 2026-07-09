//! Tests for the placeholder passthrough behavior of `process_cgp_errors`.
//!
//! These exercise the stage the way the design intends — from a *serialized* fixture,
//! with no compiler and no `cargo-cgp` process in the loop — which is what the stateless
//! signature buys. `tests/fixtures/sample_diagnostics.json` is a real rustc JSON
//! diagnostic stream (two errors and two failure-notes), captured once and committed. As
//! the placeholder is replaced by real analysis, this is where the fuller snapshot suite
//! grows: one fixture per error class, each asserting the transformed output.

use cargo_cgp_error_processing::cargo_metadata::diagnostic::Diagnostic;
use cargo_cgp_error_processing::process_cgp_errors;

/// Deserialize the committed fixture into the diagnostics the processor consumes.
fn sample_diagnostics() -> Vec<Diagnostic> {
    let json = include_str!("fixtures/sample_diagnostics.json");
    serde_json::from_str(json).expect("deserializing the sample diagnostics fixture")
}

#[test]
fn passthrough_preserves_every_diagnostic() {
    let input = sample_diagnostics();
    assert_eq!(
        input.len(),
        4,
        "fixture should carry the four captured diagnostics"
    );

    let output = process_cgp_errors(&input);

    // The placeholder passes every diagnostic through unchanged: same count, same order,
    // same underlying diagnostic. When real processing lands this assertion changes —
    // the output count will differ from the input as cascades collapse.
    assert_eq!(output.len(), input.len());
    for (processed, original) in output.iter().zip(&input) {
        assert_eq!(&processed.diagnostic, original);
    }
}

#[test]
fn rendered_matches_the_underlying_diagnostic() {
    let input = sample_diagnostics();
    let output = process_cgp_errors(&input);

    // The render stage prints `rendered` to reproduce rustc's own pretty output, so the
    // pass-through must expose exactly what rustc rendered.
    for (processed, original) in output.iter().zip(&input) {
        assert_eq!(processed.rendered(), original.rendered.as_deref());
    }
}
