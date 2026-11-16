//! Reading a diagnostic's place in the canonical order, and sorting by it.
//!
//! The order itself is rustc-free ([`cargo_cgp_error_processing::order`]); this module reads its
//! key off a `DiagInner`: the primary span's file and offsets within that file, the code, and the
//! main message.

use cargo_cgp_error_processing::byte_range::ByteRange;
use cargo_cgp_error_processing::order::{OrderKey, SpanPlace, sort_by_order_key};
use rustc_errors::DiagInner;
use rustc_span::source_map::SourceMap;

use crate::emitter::edit::main_message_text;

/// The place of `diag` in the canonical order, read against `source_map`. A dummy primary span
/// counts as none.
pub fn order_key(diag: &DiagInner, source_map: &SourceMap) -> OrderKey {
    let place = diag
        .span
        .primary_span()
        .filter(|span| !span.is_dummy())
        .map(|span| {
            let start = source_map.lookup_byte_offset(span.lo());
            let lo = start.pos.0;
            let hi = span.hi().0.saturating_sub(start.sf.start_pos.0).max(lo);
            SpanPlace {
                file: start.sf.name.prefer_local_unconditionally().to_string(),
                range: ByteRange { lo, hi },
            }
        });
    let message = match main_message_text(diag) {
        Some(text) => text.to_owned(),
        None => format!("{:?}", diag.messages.first()),
    };
    OrderKey {
        place,
        code: diag.code.map(|code| code.to_string()),
        message,
    }
}

/// `items` in the canonical order of the diagnostic `diag_of` gives each, items whose diagnostics
/// tie keeping their order.
pub fn sort_canonically<T>(
    items: Vec<T>,
    diag_of: impl Fn(&T) -> &DiagInner,
    source_map: &SourceMap,
) -> Vec<T> {
    sort_by_order_key(items, |item| order_key(diag_of(item), source_map))
}
