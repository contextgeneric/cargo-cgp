//! Tests for rebuilding rustc's "detailed explanations" footer after the flush.
//!
//! rustc builds the footer from every error code it registered, a superset of what the emitter
//! shows once it drops re-reports and cascades. The properties come from the driver document
//! (`cgp-knowledge-base/cargo-cgp/implementation/driver/program.md`): the kept codes are the footer's own
//! codes that a shown diagnostic still carries, a footer whose codes all survive is rewritten
//! identically, and a footer that names no code the rebuild can read is left exactly as written.

use cargo_cgp_driver::emitter::rebuild_explain_footers;
use rustc_errors::codes::{E0277, E0308, E0599};
use rustc_errors::{DiagInner, DiagMessage, ErrCode, Level};

const LIST_LINE: &str = "Some errors have detailed explanations: E0277, E0308, E0599.";
const POINTER_LINE: &str = "For more information about an error, try `rustc --explain E0277`.";

/// An error carrying `code`.
fn error(code: ErrCode) -> DiagInner {
    let mut diag = DiagInner::new(Level::Error, "an error");
    diag.code = Some(code);
    diag
}

/// A footer line, as rustc emits it.
fn footer(text: &str) -> DiagInner {
    DiagInner::new(Level::FailureNote, text.to_owned())
}

/// The text of each footer line left in `diags`.
fn footer_lines(diags: &[DiagInner]) -> Vec<String> {
    diags
        .iter()
        .filter(|diag| diag.level() == Level::FailureNote)
        .filter_map(|diag| match diag.messages.first() {
            Some((DiagMessage::Str(text), _)) => Some(text.to_string()),
            _ => None,
        })
        .collect()
}

/// Rebuild the footer of `errors` followed by rustc's two footer lines, and return its lines.
fn rebuilt(errors: Vec<DiagInner>) -> Vec<String> {
    let mut diags = errors;
    diags.push(footer(LIST_LINE));
    diags.push(footer(POINTER_LINE));
    rebuild_explain_footers(&mut diags);
    footer_lines(&diags)
}

// Property: for all lists in which every code the footer names is carried by a shown error, the
//   rebuilt footer is the footer rustc wrote.
#[test]
fn a_footer_whose_codes_all_survive_is_unchanged() {
    assert_eq!(
        rebuilt(vec![error(E0277), error(E0308), error(E0599)]),
        vec![LIST_LINE.to_owned(), POINTER_LINE.to_owned()],
        "every code survives, so the footer is as rustc wrote it",
    );
}

// Property: the kept codes are the footer's codes that a shown error carries, in the footer's
//   order; a code the emitter dropped every error of leaves the list.
#[test]
fn a_dropped_code_leaves_the_list() {
    assert_eq!(
        rebuilt(vec![error(E0599), error(E0277)]),
        vec![
            "Some errors have detailed explanations: E0277, E0599.".to_owned(),
            POINTER_LINE.to_owned(),
        ],
        "only `E0308` lost its errors, so only it leaves the footer",
    );
}

// Property: with one kept code, the list line goes and the pointer line names that code in the
//   singular form rustc gives one code.
#[test]
fn one_surviving_code_keeps_only_the_pointer_line() {
    assert_eq!(
        rebuilt(vec![error(E0308)]),
        vec!["For more information about this error, try `rustc --explain E0308`.".to_owned()],
        "one code is pointed at alone",
    );
}

// Property: when no shown error carries a code the footer names, both footer lines go.
#[test]
fn no_surviving_code_drops_the_footer() {
    let mut diags = vec![
        DiagInner::new(Level::Error, "an error without a code"),
        footer(LIST_LINE),
        footer(POINTER_LINE),
    ];
    rebuild_explain_footers(&mut diags);
    assert!(
        footer_lines(&diags).is_empty(),
        "nothing is left to explain: {:?}",
        footer_lines(&diags),
    );
    assert_eq!(diags.len(), 1, "the error itself stays");
}

// Concern: a footer line rustc reworded would parse as naming no code, and rebuilding it from
//   nothing would delete it.
// Handled: the rebuild declines when it reads no code from any footer line.
// Reaches: the line opens as a footer line does, so it is recognized as one, but names no
//   `--explain` code to read.
#[test]
fn a_footer_naming_no_readable_code_is_left_as_written() {
    let reworded = "For more information about an error, see the error index.";
    let mut diags = vec![error(E0277), footer(reworded)];
    rebuild_explain_footers(&mut diags);
    assert_eq!(
        footer_lines(&diags),
        vec![reworded.to_owned()],
        "an unreadable footer stays as rustc wrote it",
    );
}
