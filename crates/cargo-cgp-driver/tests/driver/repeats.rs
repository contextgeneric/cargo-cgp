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

/// The source the carets point into. On the first line, `Shape` is bytes 8 to 13, `from_tag` 15
/// to 23, the whole call 8 to 36, and `Circle` 29 to 35; `other` is bytes 46 to 51, on the second.
const SOURCE: &str = "let _ = Shape::from_tag(tag, Circle);\nlet _ = other();\n";
const SHAPE: (u32, u32) = (8, 13);
const FROM_TAG: (u32, u32) = (15, 23);
const CALL: (u32, u32) = (8, 36);
const CIRCLE: (u32, u32) = (29, 35);
const OTHER: (u32, u32) = (46, 51);

const MESSAGE: &str = "the trait bound `Shape: FromTag<CircleTag>` is not satisfied";

/// Run `test` with a source map holding [`SOURCE`] and a function that spans a byte range of it.
fn with_source(test: impl FnOnce(&SourceMap, &dyn Fn((u32, u32)) -> Span)) {
    create_default_session_globals_then(|| {
        let map = SourceMap::new(FilePathMapping::empty());
        let file = map.new_source_file(FileName::Custom("main.rs".to_owned()), SOURCE.to_owned());
        let start = file.start_pos;
        let span =
            |(lo, hi): (u32, u32)| Span::with_root_ctxt(start + BytePos(lo), start + BytePos(hi));
        test(&map, &span);
    });
}

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

#[test]
fn a_repeat_at_another_caret_on_the_line_is_merged_and_labelled() {
    with_source(|map, span| {
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

#[test]
fn a_repeat_at_a_caret_the_first_block_covers_adds_no_label() {
    with_source(|map, span| {
        // The same caret, as when rustc checks one requirement for each item a macro generated,
        // and the whole call around a caret on its method name.
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

#[test]
fn the_same_error_on_another_line_is_kept() {
    with_source(|map, span| {
        let diags = vec![
            error(Some(E0277), MESSAGE, span(SHAPE)),
            error(Some(E0277), MESSAGE, span(OTHER)),
        ];

        assert_eq!(merge_repeats(diags, map).len(), 2);
    });
}

#[test]
fn an_error_that_differs_in_its_code_its_message_or_its_level_is_kept() {
    with_source(|map, span| {
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

#[test]
fn an_error_without_a_code_is_kept() {
    with_source(|map, span| {
        let diags = vec![
            error(None, MESSAGE, span(SHAPE)),
            error(None, MESSAGE, span(CIRCLE)),
        ];

        assert_eq!(merge_repeats(diags, map).len(), 2);
    });
}

#[test]
fn an_error_with_an_inline_message_is_kept() {
    with_source(|map, span| {
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
