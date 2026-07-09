//! Tests for the individual preprocessors.
//!
//! Each builds a `CgpDiagnostic` from a minimal diagnostic whose `rendered` (and
//! `message`) text is the case under test, runs one preprocessor, and asserts on the
//! rewritten text and the `has_cgp_error` flag.

use cargo_cgp_error_processing::cargo_metadata::diagnostic::Diagnostic;
use cargo_cgp_error_processing::{CgpDiagnostic, resugar_symbol, strip_cgp_prefixes};

/// Build a `CgpDiagnostic` whose message and rendered text are both `text`.
fn diagnostic(text: &str) -> CgpDiagnostic {
    let value = serde_json::json!({
        "message": text,
        "code": null,
        "level": "error",
        "spans": [],
        "children": [],
        "rendered": text,
    });
    let diagnostic: Diagnostic =
        serde_json::from_value(value).expect("building a diagnostic from JSON");
    CgpDiagnostic::wrap(diagnostic)
}

#[test]
fn strips_known_cgp_prefixes_and_flags_cgp() {
    let output = strip_cgp_prefixes(diagnostic(
        "HasField<cgp::prelude::Symbol<...>>, cgp::cgp_core::Foo, cgp::cgp_extra::Bar",
    ));
    assert_eq!(
        output.rendered().unwrap(),
        "HasField<Symbol<...>>, Foo, Bar"
    );
    assert!(output.has_cgp_error);
}

#[test]
fn strip_leaves_non_cgp_text_untouched() {
    let text = "the trait bound `Rectangle: Greeter` is not satisfied";
    let output = strip_cgp_prefixes(diagnostic(text));
    assert_eq!(output.rendered().unwrap(), text);
    assert!(!output.has_cgp_error);
}

#[test]
fn resugars_an_exact_symbol_and_flags_cgp() {
    // `height` — six ASCII characters, so the length is 6. The surrounding `HasField<…>`
    // must survive with its own closing `>`.
    let output = resugar_symbol(diagnostic(
        "HasField<Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<'h', Chars<'t', Nil>>>>>>>>",
    ));
    assert_eq!(output.rendered().unwrap(), "HasField<Symbol!(\"height\")>");
    assert!(output.has_cgp_error);
}

#[test]
fn resugars_the_empty_symbol() {
    let output = resugar_symbol(diagnostic("Symbol<0, Nil>"));
    assert_eq!(output.rendered().unwrap(), "Symbol!(\"\")");
    assert!(output.has_cgp_error);
}

#[test]
fn resugar_skips_a_wrong_length() {
    // Declared length 3 but only two characters — not an exact match, so left alone.
    let text = "Symbol<3, Chars<'x', Chars<'y', Nil>>>";
    let output = resugar_symbol(diagnostic(text));
    assert_eq!(output.rendered().unwrap(), text);
    assert!(!output.has_cgp_error);
}

#[test]
fn resugar_skips_a_foreign_symbol() {
    // A type named `Symbol` with a non-`Chars` payload must not be rewritten.
    let text = "Symbol<2, SomeOtherType>";
    let output = resugar_symbol(diagnostic(text));
    assert_eq!(output.rendered().unwrap(), text);
    assert!(!output.has_cgp_error);
}

#[test]
fn resugar_leaves_other_symbol_uses_alone() {
    let text = "the trait `Symbolic` is not implemented";
    let output = resugar_symbol(diagnostic(text));
    assert_eq!(output.rendered().unwrap(), text);
    assert!(!output.has_cgp_error);
}
