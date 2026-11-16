//! Tests for reading a diagnostic's place in the canonical order
//! ([`cargo_cgp_driver::emitter::order_key`]) and sorting by it.

use cargo_cgp_driver::emitter::{order_key, sort_canonically};
use cargo_cgp_error_processing::byte_range::ByteRange;
use rustc_errors::codes::E0277;
use rustc_errors::{DiagInner, Level, MultiSpan};
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
        let sorted: Vec<String> = sort_canonically(arrived, |diag| diag, map)
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
