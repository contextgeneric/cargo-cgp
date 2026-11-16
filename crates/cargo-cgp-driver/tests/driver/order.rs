//! Tests for reading a diagnostic's place in the canonical order
//! ([`cargo_cgp_driver::emitter::order_key`]) and sorting by it.

use cargo_cgp_driver::emitter::{order_key, sort_canonically};
use cargo_cgp_error_processing::byte_range::ByteRange;
use cargo_cgp_error_processing::order::Spanless;
use rustc_errors::codes::E0277;
use rustc_errors::{DiagInner, DiagMessage, Level, MultiSpan, Style, Subdiag, Sublevel};
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
