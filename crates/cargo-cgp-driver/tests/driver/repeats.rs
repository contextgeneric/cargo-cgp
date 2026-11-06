//! Tests for merging a diagnostic rustc reports twice on one line into its first report.
//!
//! Each test builds its diagnostics over one in-memory source file, so their carets fall on known
//! lines, and checks which blocks survive and how the first one is labelled.

use cargo_cgp_driver::emitter::merge_repeats;
use cargo_cgp_error_processing::COALESCED_SITE_LABEL;
use rustc_errors::codes::{E0277, E0308};
use rustc_errors::{DiagInner, DiagMessage, ErrCode, Level, MultiSpan};
use rustc_span::source_map::{FilePathMapping, SourceMap};
use rustc_span::{BytePos, FileName, Span, create_default_session_globals_then};

use crate::driver::source;

/// The source the carets point into. On the first line, `Shape` is bytes 8 to 13, `from_tag` 15
/// to 23, the whole call 8 to 36, and `Circle` 29 to 35; `other` is bytes 46 to 51, on the second.
const SOURCE: &str = "let _ = Shape::from_tag(tag, Circle);\nlet _ = other();\n";
const SHAPE: (u32, u32) = (8, 13);
const FROM_TAG: (u32, u32) = (15, 23);
const CALL: (u32, u32) = (8, 36);
const CIRCLE: (u32, u32) = (29, 35);
const OTHER: (u32, u32) = (46, 51);

const MESSAGE: &str = "the trait bound `Shape: FromTag<CircleTag>` is not satisfied";

/// An error with `code` and `message` at the primary `caret`.
fn error(code: Option<ErrCode>, message: &str, caret: Span) -> DiagInner {
    let mut diag = DiagInner::new(Level::Error, message.to_owned());
    diag.code = code;
    diag.span = MultiSpan::from_span(caret);
    diag
}

/// The spans of `diag` labelled [`COALESCED_SITE_LABEL`].
fn also_required(diag: &DiagInner) -> Vec<Span> {
    diag.span
        .span_labels()
        .into_iter()
        .filter(|label| {
            matches!(&label.label, Some(DiagMessage::Str(text)) if text == COALESCED_SITE_LABEL)
        })
        .map(|label| label.span)
        .collect()
}

// Property: a repeat of an earlier diagnostic (the same level, code, and plain-string message,
//   with a primary caret on the same line of the same file) is merged into it, and a caret the
//   earlier block does not cover becomes a secondary caret labelled `COALESCED_SITE_LABEL`.
#[test]
fn a_repeat_at_another_caret_on_the_line_is_merged_and_labelled() {
    source::with_source(SOURCE, |map, span| {
        let diags = vec![
            error(Some(E0277), MESSAGE, span(SHAPE)),
            error(Some(E0277), MESSAGE, span(CIRCLE)),
        ];

        let merged = merge_repeats(diags, map);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].span.primary_span(), Some(span(SHAPE)));
        assert_eq!(also_required(&merged[0]), vec![span(CIRCLE)]);
    });
}

// Property: a repeat whose caret overlaps a span of the earlier block is merged without a label:
//   the same caret, as when rustc checks a requirement once for each item a macro generated, and
//   the whole call around an earlier caret on its method name.
#[test]
fn a_repeat_at_a_caret_the_first_block_covers_adds_no_label() {
    source::with_source(SOURCE, |map, span| {
        for (first, repeat) in [(SHAPE, SHAPE), (FROM_TAG, CALL)] {
            let diags = vec![
                error(Some(E0277), MESSAGE, span(first)),
                error(Some(E0277), MESSAGE, span(repeat)),
            ];

            let merged = merge_repeats(diags, map);

            assert_eq!(merged.len(), 1);
            assert_eq!(also_required(&merged[0]), Vec::<Span>::new());
        }
    });
}

// Property: the same error with its caret on another line is not a repeat, so both are kept.
#[test]
fn the_same_error_on_another_line_is_kept() {
    source::with_source(SOURCE, |map, span| {
        let diags = vec![
            error(Some(E0277), MESSAGE, span(SHAPE)),
            error(Some(E0277), MESSAGE, span(OTHER)),
        ];

        assert_eq!(merge_repeats(diags, map).len(), 2);
    });
}

// Property: an error on the same line that differs from the earlier one in its code, its message,
//   or its level is not a repeat, so both are kept.
#[test]
fn an_error_that_differs_in_its_code_its_message_or_its_level_is_kept() {
    source::with_source(SOURCE, |map, span| {
        let mut warning = DiagInner::new(Level::Warning(None), MESSAGE.to_owned());
        warning.code = Some(E0277);
        warning.span = MultiSpan::from_span(span(CIRCLE));
        for repeat in [
            error(Some(E0308), MESSAGE, span(CIRCLE)),
            error(Some(E0277), "mismatched types", span(CIRCLE)),
            warning,
        ] {
            let diags = vec![error(Some(E0277), MESSAGE, span(SHAPE)), repeat];

            assert_eq!(merge_repeats(diags, map).len(), 2);
        }
    });
}

// Property (assumed from the module doc, whose repeats carry rustc's code): errors without a code
//   are never merged, even when they repeat on one line.
#[test]
fn an_error_without_a_code_is_kept() {
    source::with_source(SOURCE, |map, span| {
        let diags = vec![
            error(None, MESSAGE, span(SHAPE)),
            error(None, MESSAGE, span(CIRCLE)),
        ];

        assert_eq!(merge_repeats(diags, map).len(), 2);
    });
}

// Property: errors whose main message is not a plain string (here an inline message) are never
//   merged, since the merge compares plain-string messages only.
#[test]
fn an_error_with_an_inline_message_is_kept() {
    source::with_source(SOURCE, |map, span| {
        let inline = |caret| {
            let mut diag = DiagInner::new(Level::Error, DiagMessage::Inline(MESSAGE.into()));
            diag.code = Some(E0277);
            diag.span = MultiSpan::from_span(caret);
            diag
        };
        let diags = vec![inline(span(SHAPE)), inline(span(CIRCLE))];

        assert_eq!(merge_repeats(diags, map).len(), 2);
    });
}

// Property: the same error on the same line number of another file is not a repeat, so both are
//   kept.
#[test]
fn the_same_error_on_the_same_line_of_another_file_is_kept() {
    create_default_session_globals_then(|| {
        let map = SourceMap::new(FilePathMapping::empty());
        let shape_at = |name: &str| {
            let file = map.new_source_file(FileName::Custom(name.to_owned()), SOURCE.to_owned());
            let (lo, hi) = SHAPE;
            Span::with_root_ctxt(file.start_pos + BytePos(lo), file.start_pos + BytePos(hi))
        };
        let diags = vec![
            error(Some(E0277), MESSAGE, shape_at("main.rs")),
            error(Some(E0277), MESSAGE, shape_at("lib.rs")),
        ];

        assert_eq!(merge_repeats(diags, &map).len(), 2);
    });
}
