//! Tests for reading a diagnostic's place in the canonical order
//! ([`cargo_cgp_driver::emitter::order_key`]) and sorting by it.

use cargo_cgp_driver::emitter::{order_key, sort_canonically};
use cargo_cgp_error_processing::byte_range::ByteRange;
use cargo_cgp_error_processing::order::Spanless;
use rustc_errors::codes::E0277;
use rustc_errors::{
    Applicability, CodeSuggestion, DiagInner, DiagMessage, Level, MultiSpan, Style, Subdiag,
    Sublevel, Substitution, SubstitutionPart, SuggestionStyle, Suggestions,
};
use rustc_span::source_map::{FilePathMapping, SourceMap};
use rustc_span::{BytePos, DUMMY_SP, FileName, Span, create_default_session_globals_then};

/// An `E0277` with `message` at the primary `caret`.
fn error(message: &str, caret: Span) -> DiagInner {
    let mut diag = DiagInner::new(Level::Error, message.to_owned());
    diag.code = Some(E0277);
    diag.span = MultiSpan::from_span(caret);
    diag
}

/// Run `test` with a source map holding a dependency's file and then `main.rs`, and a function
/// that spans a byte range of `main.rs`.
fn with_two_files(test: impl FnOnce(&SourceMap, &dyn Fn(u32, u32) -> Span)) {
    create_default_session_globals_then(|| {
        let map = SourceMap::new(FilePathMapping::empty());
        map.new_source_file(FileName::Custom("dep.rs".to_owned()), "x".repeat(40));
        let file = map.new_source_file(
            FileName::Custom("main.rs".to_owned()),
            "let _ = call(arg);\n".to_owned(),
        );
        let start = file.start_pos;
        let span =
            |lo: u32, hi: u32| Span::with_root_ctxt(start + BytePos(lo), start + BytePos(hi));
        test(&map, &span);
    });
}

// Concern: a dependency's file entering the source map first shifts every global position in
//   `main.rs`, which under threads would make the order depend on which span was decoded first.
// Handled: `order_key` reads the offsets from the start of the span's own file.
// Property: for all spans in a file, the key's range is the span's offsets within that file.
#[test]
fn the_key_reads_offsets_within_the_spans_file() {
    with_two_files(|map, span| {
        let key = order_key(&error("m", span(8, 17)), map);
        let place = key.place.expect("a span has a place");
        assert_eq!(place.range, ByteRange { lo: 8, hi: 17 });
        assert!(place.file.contains("main.rs"), "{}", place.file);
        assert_eq!(key.code.as_deref(), Some("E0277"));
        assert_eq!(key.message, "m");
    });
}

// Property: `sort_canonically` puts a diagnostic at an inner span before the one at the span around
//   it, whatever order they come in, and a diagnostic with a dummy span after both.
#[test]
fn sorting_puts_the_inner_report_first_and_the_spanless_last() {
    with_two_files(|map, span| {
        let arrived = vec![
            error("spanless", DUMMY_SP),
            error("the whole call", span(8, 17)),
            error("the argument", span(13, 16)),
        ];
        let sorted: Vec<String> =
            sort_canonically(arrived, |diag| diag, map, Spanless::InArrivalOrder)
                .iter()
                .map(|diag| format!("{:?}", diag.messages[0].0))
                .collect();
        assert_eq!(
            sorted,
            [
                r#"Str("the argument")"#,
                r#"Str("the whole call")"#,
                r#"Str("spanless")"#
            ]
        );
    });
}

/// `diag` with a note child reading `note`.
fn with_note(mut diag: DiagInner, note: &str) -> DiagInner {
    diag.children.push(Subdiag {
        level: Sublevel::Note,
        messages: vec![(DiagMessage::from(note.to_owned()), Style::NoStyle)],
        span: MultiSpan::new(),
    });
    diag
}

// Concern: two diagnostics at one span with one code and message, which differ only in a child,
//   tie on place, code, and message, so under threads their order would follow arrival, as
//   `merge_repeats` and the flush then would.
// Handled: `order_key` reads the children into the detail.
// Property: for all orders of arrival, `sort_canonically` gives such diagnostics in one order.
#[test]
fn diagnostics_that_differ_in_a_child_sort_the_same_in_either_arrival_order() {
    with_two_files(|map, span| {
        let first = || with_note(error("m", span(8, 17)), "required for `A`");
        let second = || with_note(error("m", span(8, 17)), "required for `B`");
        let notes = |sorted: Vec<DiagInner>| -> Vec<String> {
            sorted
                .iter()
                .map(|diag| format!("{:?}", diag.children[0].messages[0].0))
                .collect()
        };
        let one = notes(sort_canonically(
            vec![first(), second()],
            |d| d,
            map,
            Spanless::InArrivalOrder,
        ));
        let other = notes(sort_canonically(
            vec![second(), first()],
            |d| d,
            map,
            Spanless::InArrivalOrder,
        ));
        assert_eq!(one, other);
        assert_ne!(order_key(&first(), map), order_key(&second(), map));
    });
}

/// The main messages of `diags`, in order.
fn messages(diags: &[DiagInner]) -> Vec<String> {
    diags
        .iter()
        .map(|diag| format!("{:?}", diag.messages[0].0))
        .collect()
}

// Concern: a batch of `analysis` can hold two diagnostics with a dummy span, which rustc's threads
//   emit in either order, and the batch would then be processed and printed in that order.
// Handled: the batch sorts with `Spanless::ByContent`, which reads the message into the order.
// Property: for both arrival orders of two diagnostics with a dummy span, `sort_canonically` with
//   `Spanless::ByContent` gives one order; with `Spanless::InArrivalOrder`, as the flush sorts,
//   each keeps its arrival order, so rustc's closing lines stay last in theirs.
#[test]
fn spanless_diagnostics_sort_by_content_in_a_batch_and_keep_their_order_at_the_flush() {
    with_two_files(|map, _| {
        let a = || error("an overflow", DUMMY_SP);
        let b = || error("a cycle", DUMMY_SP);
        let by_content = |arrived: Vec<DiagInner>| {
            messages(&sort_canonically(arrived, |d| d, map, Spanless::ByContent))
        };
        assert_eq!(by_content(vec![a(), b()]), by_content(vec![b(), a()]));
        assert_eq!(
            messages(&sort_canonically(
                vec![a(), b()],
                |d| d,
                map,
                Spanless::InArrivalOrder
            )),
            [r#"Str("an overflow")"#, r#"Str("a cycle")"#]
        );
    });
}

/// The order of the two variants `make` builds, sorted in each arrival order, as the indices of
/// the variants, with whether their keys differ.
fn order_of(map: &SourceMap, make: impl Fn(bool) -> DiagInner) -> (Vec<bool>, Vec<bool>, bool) {
    let first = |sorted: Vec<(bool, DiagInner)>| sorted.into_iter().map(|(v, _)| v).collect();
    let sort = |items: Vec<(bool, DiagInner)>| {
        sort_canonically(items, |(_, diag)| diag, map, Spanless::InArrivalOrder)
    };
    let one = first(sort(vec![(false, make(false)), (true, make(true))]));
    let other = first(sort(vec![(true, make(true)), (false, make(false))]));
    (
        one,
        other,
        order_key(&make(false), map) != order_key(&make(true), map),
    )
}

// Concern: two diagnostics at one span with one code and message print differently when they
//   differ only in their level, in a child's secondary label, or in a suggestion's style or
//   applicability, which the detail did not read, so a stable sort kept them in arrival order,
//   which varies under threads.
// Handled: `order_key`'s detail reads each of the four.
// Property: for each of the four differences, the two diagnostics have different keys, and
//   `sort_canonically` gives them one order in both arrival orders.
#[test]
fn diagnostics_that_differ_only_in_a_printed_part_sort_the_same_in_either_order() {
    with_two_files(|map, span| {
        let level = |warning: bool| {
            let level = if warning {
                Level::Warning(None)
            } else {
                Level::Error
            };
            let mut diag = DiagInner::new(level, "m");
            diag.code = Some(E0277);
            diag.span = MultiSpan::from_span(span(8, 17));
            diag
        };
        let child_label = |labelled: bool| {
            let mut diag = with_note(error("m", span(8, 17)), "required here");
            let mut child_span = MultiSpan::from_span(span(8, 12));
            if labelled {
                child_span.push_span_label(span(13, 16), "this argument");
            }
            diag.children[0].span = child_span;
            diag
        };
        let suggestion = |style: SuggestionStyle, applicability: Applicability| {
            let mut diag = error("m", span(8, 17));
            diag.suggestions = Suggestions::Enabled(vec![CodeSuggestion {
                substitutions: vec![Substitution {
                    parts: vec![SubstitutionPart {
                        span: span(13, 16),
                        snippet: "x".to_owned(),
                    }],
                }],
                msg: DiagMessage::from("use `x`"),
                style,
                applicability,
            }]);
            diag
        };
        let style = |hidden: bool| {
            let style = if hidden {
                SuggestionStyle::HideCodeInline
            } else {
                SuggestionStyle::ShowCode
            };
            suggestion(style, Applicability::MachineApplicable)
        };
        let applicability = |maybe: bool| {
            let applicability = if maybe {
                Applicability::MaybeIncorrect
            } else {
                Applicability::MachineApplicable
            };
            suggestion(SuggestionStyle::ShowCode, applicability)
        };
        for (part, (one, other, differ)) in [
            ("level", order_of(map, level)),
            ("child label", order_of(map, child_label)),
            ("suggestion style", order_of(map, style)),
            ("applicability", order_of(map, applicability)),
        ] {
            assert!(differ, "{part}: the keys differ");
            assert_eq!(one, other, "{part}: one order in both arrival orders");
        }
    });
}
