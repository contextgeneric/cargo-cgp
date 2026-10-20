//! Merging a diagnostic the compiler reports twice.
//!
//! rustc reports some CGP errors a second time with the same code and message: at the same caret,
//! with a note renamed or dropped, when it checks one requirement once for each item a macro
//! generated, or at a second caret on the same line, when it checks a call's bound once for the
//! call and once for an argument. The two blocks state one mistake, so the later one is merged into
//! the earlier, whose notes are the fuller. A caret the earlier block already covers, such as the
//! whole call around its method name, adds nothing; one elsewhere on the line is kept as a secondary
//! caret labelled [`COALESCED_SITE_LABEL`], so no location is lost.

use cargo_cgp_error_processing::COALESCED_SITE_LABEL;
use rustc_errors::DiagInner;
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

use crate::emitter::edit::main_message_text;

/// `diags` with each repeat of an earlier diagnostic merged into it: the same level, code, and
/// plain-string message, with a primary caret on the same line of the same file.
pub fn merge_repeats(diags: Vec<DiagInner>, source_map: &SourceMap) -> Vec<DiagInner> {
    let mut kept: Vec<DiagInner> = Vec::with_capacity(diags.len());
    for diag in diags {
        let Some(caret) = diag.span.primary_span() else {
            kept.push(diag);
            continue;
        };
        let earlier = kept.iter_mut().find(|earlier| {
            earlier.level() == diag.level()
                && earlier.code.is_some()
                && earlier.code == diag.code
                && main_message_text(earlier).is_some()
                && main_message_text(earlier) == main_message_text(&diag)
                && earlier
                    .span
                    .primary_span()
                    .is_some_and(|first| same_line(source_map, first, caret))
        });
        let Some(earlier) = earlier else {
            kept.push(diag);
            continue;
        };
        let shown = earlier
            .span
            .span_labels()
            .iter()
            .any(|label| label.span.overlaps(caret));
        if !shown {
            earlier.span.push_span_label(caret, COALESCED_SITE_LABEL);
        }
    }
    kept
}

/// Whether `a` and `b` start on the same line of the same file.
fn same_line(source_map: &SourceMap, a: Span, b: Span) -> bool {
    let (a, b) = (
        source_map.lookup_char_pos(a.lo()),
        source_map.lookup_char_pos(b.lo()),
    );
    a.file.name == b.file.name && a.line == b.line
}
