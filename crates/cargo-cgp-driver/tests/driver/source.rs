//! An in-memory source file for the tests that build spans.

use rustc_span::source_map::{FilePathMapping, SourceMap};
use rustc_span::{BytePos, FileName, Span, create_default_session_globals_then};

/// Run `test` with a source map holding `source` as `main.rs` and a function that spans a byte
/// range of it.
pub fn with_source(source: &str, test: impl FnOnce(&SourceMap, &dyn Fn((u32, u32)) -> Span)) {
    create_default_session_globals_then(|| {
        let map = SourceMap::new(FilePathMapping::empty());
        let file = map.new_source_file(FileName::Custom("main.rs".to_owned()), source.to_owned());
        let start = file.start_pos;
        let span =
            |(lo, hi): (u32, u32)| Span::with_root_ctxt(start + BytePos(lo), start + BytePos(hi));
        test(&map, &span);
    });
}
