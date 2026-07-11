//! Tests for the individual preprocessors.
//!
//! Each builds a `CgpDiagnostic` from a minimal diagnostic whose `rendered` (and
//! `message`) text is the case under test, runs one preprocessor, and asserts on the
//! rewritten text and the `has_cgp_error` flag.

use cargo_cgp_error_processing::cargo_metadata::diagnostic::Diagnostic;
use cargo_cgp_error_processing::{
    CgpDiagnostic, CgpDiagnosticDetail, extract_missing_fields, mark_cgp_header, resugar_symbol,
    strip_cgp_prefixes,
};

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

#[test]
fn missing_field_with_inline_landmark_absorbs_it() {
    // A single missing field: the "but trait … is implemented for it" landmark follows
    // the clause inline, so it is absorbed into the rewrite.
    let output = extract_missing_fields(diagnostic(
        "help: the trait `HasField<Symbol!(\"height\")>` is not implemented for `Rectangle`\n\
         \x20     but trait `HasField<Symbol!(\"width\")>` is implemented for it\n\
         \x20 --> src/main.rs:45:10",
    ));
    assert_eq!(
        output.rendered().unwrap(),
        "help: [CGP0001] missing field `height` in `Rectangle`\n  --> src/main.rs:45:10"
    );
    assert!(output.has_cgp_error);
    assert_eq!(
        output.details,
        [CgpDiagnosticDetail::MissingField {
            field_name: "height".to_owned(),
            context: "Rectangle".to_owned(),
        }]
    );
}

#[test]
fn missing_field_with_separate_impls_note_is_recognized() {
    // Several other fields: the landmark is a separate `implements trait HasField` note,
    // not inline. It still classifies as a single missing field; the note is left as-is.
    let output = extract_missing_fields(diagnostic(
        "help: the trait `HasField<Symbol!(\"height\")>` is not implemented for `Rectangle`\n\
         \x20 --> src/main.rs:59:1\n\
         help: `Rectangle` implements trait `HasField<Tag>`",
    ));
    assert_eq!(
        output.rendered().unwrap(),
        "help: [CGP0001] missing field `height` in `Rectangle`\n  --> src/main.rs:59:1\n\
         help: `Rectangle` implements trait `HasField<Tag>`"
    );
    assert!(output.has_cgp_error);
    assert_eq!(
        output.details,
        [CgpDiagnosticDetail::MissingField {
            field_name: "height".to_owned(),
            context: "Rectangle".to_owned(),
        }]
    );
}

#[test]
fn missing_derive_when_no_impls_present() {
    // No landmark at all: the context implements HasField for nothing, so the whole
    // derive is missing and the message points at the derive, not a single field.
    let output = extract_missing_fields(diagnostic(
        "help: the trait `HasField<Symbol!(\"width\")>` is not implemented for `Rectangle`\n\
         \x20 --> src/main.rs:41:1",
    ));
    assert_eq!(
        output.rendered().unwrap(),
        "help: [CGP0002] `#[derive(HasField)]` is required to access field `width` in `Rectangle`\n\
         \x20 --> src/main.rs:41:1"
    );
    assert!(output.has_cgp_error);
    assert_eq!(
        output.details,
        [CgpDiagnosticDetail::MissingDeriveHasField {
            field_name: "width".to_owned(),
            context: "Rectangle".to_owned(),
        }]
    );
}

#[test]
fn missing_field_leaves_unrelated_diagnostics_alone() {
    let text = "error[E0277]: the trait bound `Foo: Bar` is not satisfied";
    let output = extract_missing_fields(diagnostic(text));
    assert_eq!(output.rendered().unwrap(), text);
    assert!(!output.has_cgp_error);
    assert!(output.details.is_empty());
}

#[test]
fn header_mark_recognizes_a_wiring_rename_and_marks_the_header() {
    // The driver's rename phrasing ("consumer trait bound") flags the diagnostic on its own,
    // and its `error[E0277]:` header becomes `CGP[E0277]:` with the code kept.
    let output = mark_cgp_header(diagnostic(
        "error[E0277]: the consumer trait bound `Rectangle: CanCalculateArea` is not satisfied",
    ));
    assert_eq!(
        output.rendered().unwrap(),
        "CGP[E0277]: the consumer trait bound `Rectangle: CanCalculateArea` is not satisfied"
    );
    assert!(output.has_cgp_error);
}

#[test]
fn header_mark_marks_an_already_flagged_diagnostic() {
    // A diagnostic an earlier preprocessor flagged (has_cgp_error true) gets its header
    // marked even without a wiring-rename phrase.
    let mut input = diagnostic("error[E0277]: some already-recognized CGP error");
    input.has_cgp_error = true;
    let output = mark_cgp_header(input);
    assert_eq!(
        output.rendered().unwrap(),
        "CGP[E0277]: some already-recognized CGP error"
    );
}

#[test]
fn header_mark_leaves_non_cgp_diagnostics_alone() {
    let text = "error[E0277]: the trait bound `Foo: Bar` is not satisfied";
    let output = mark_cgp_header(diagnostic(text));
    assert_eq!(output.rendered().unwrap(), text);
    assert!(!output.has_cgp_error);
}

#[test]
fn header_mark_keeps_the_explain_line_and_its_code() {
    // Only the leading header is rewritten; the trailing `--explain E0277` keeps its code.
    let mut input = diagnostic(
        "error[E0277]: the consumer trait bound `Rectangle: CanCalculateArea` is not satisfied\n\
         For more information about this error, try `rustc --explain E0277`.",
    );
    input.has_cgp_error = true;
    let output = mark_cgp_header(input);
    assert_eq!(
        output.rendered().unwrap(),
        "CGP[E0277]: the consumer trait bound `Rectangle: CanCalculateArea` is not satisfied\n\
         For more information about this error, try `rustc --explain E0277`."
    );
}
