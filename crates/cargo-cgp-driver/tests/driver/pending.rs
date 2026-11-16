//! Tests for settling the buffered entries whose fate depends on another diagnostic
//! ([`cargo_cgp_driver::emitter::settle_pending`]), in either arrival order.

use cargo_cgp_driver::emitter::{BufEntry, PendingEntry, PendingKind, settle_pending};
use rustc_errors::{DiagInner, Level, MultiSpan};
use rustc_span::Span;

use crate::driver::source;

/// On the one line, the call `make()` is bytes 8 to 14, the `make()?` around it 8 to 15, and
/// `other` 16 to 21.
const SOURCE: &str = "let _ = make()? other";
const CALL: (u32, u32) = (8, 14);
const TRY: (u32, u32) = (8, 15);
const OTHER: (u32, u32) = (16, 21);

const HEADER: &str = "[CGP-E001] `App` cannot implement `CanGreet`";

fn error(message: &str, caret: Span) -> DiagInner {
    let mut diag = DiagInner::new(Level::Error, message.to_owned());
    diag.span = MultiSpan::from_span(caret);
    diag
}

fn plain(message: &str, caret: Span) -> BufEntry {
    BufEntry::Plain(Box::new(error(message, caret)))
}

fn pending(message: &str, caret: Span, kind: PendingKind) -> BufEntry {
    BufEntry::Pending(Box::new(PendingEntry {
        diag: error(message, caret),
        kind,
    }))
}

fn declined() -> PendingKind {
    PendingKind::Declined {
        headers: vec![HEADER.to_owned()],
    }
}

/// Each settled entry as `(plain?, message)`, to compare whole buffers.
fn summary(buffer: &[BufEntry]) -> Vec<(bool, String)> {
    buffer
        .iter()
        .map(|entry| match entry {
            BufEntry::Plain(diag) => (true, message(diag)),
            BufEntry::Pending(pending) => (false, message(&pending.diag)),
            BufEntry::Resolved(entry) => (false, message(&entry.diag)),
        })
        .collect()
}

fn message(diag: &DiagInner) -> String {
    diag.messages[0].0.as_str().unwrap_or_default().to_owned()
}

// Property: the postcondition of `settle_pending`, for a cascade whose span overlaps a recorded CGP
//   span: it is dropped whether it sits before or after the failure it cascades from, and the other
//   entries keep their order.
#[test]
fn drops_a_cascade_on_either_side_of_its_failure() {
    source::with_source(SOURCE, |_, span| {
        let failure = || plain("the failing call", span(CALL));
        let cascade = || pending("the `?` cascade", span(TRY), PendingKind::Cascade);
        for buffer in [vec![cascade(), failure()], vec![failure(), cascade()]] {
            let settled = settle_pending(buffer, &[span(CALL)], |_| false);
            assert_eq!(summary(&settled), [(true, "the failing call".to_owned())]);
        }
    });
}

// Property: the postcondition of `settle_pending`, for a cascade no recorded CGP span overlaps: it
//   is kept as a plain entry at its position.
#[test]
fn keeps_a_cascade_with_no_failure_on_its_expression() {
    source::with_source(SOURCE, |_, span| {
        let buffer = vec![
            pending("the `?` cascade", span(TRY), PendingKind::Cascade),
            plain("another error", span(OTHER)),
        ];
        let settled = settle_pending(buffer, &[span(OTHER)], |_| false);
        assert_eq!(
            summary(&settled),
            [
                (true, "the `?` cascade".to_owned()),
                (true, "another error".to_owned())
            ]
        );
    });
}

// Property: the postcondition of `settle_pending`, for a declined report: it is dropped exactly
//   when a resolved report recorded one of its headlines, wherever the resolved one sits, and kept
//   as a plain entry otherwise.
#[test]
fn drops_a_declined_report_only_when_its_headline_was_resolved() {
    source::with_source(SOURCE, |_, span| {
        let buffer = || {
            vec![
                pending(HEADER, span(CALL), declined()),
                plain("another error", span(OTHER)),
            ]
        };
        let resolved = settle_pending(buffer(), &[], |header| header == HEADER);
        assert_eq!(summary(&resolved), [(true, "another error".to_owned())]);

        let unresolved = settle_pending(buffer(), &[], |_| false);
        assert_eq!(
            summary(&unresolved),
            [
                (true, HEADER.to_owned()),
                (true, "another error".to_owned())
            ]
        );
    });
}
