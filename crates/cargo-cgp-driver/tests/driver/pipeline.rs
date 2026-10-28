//! Tests for the emitter's stage pipeline: the verdict chain and the step that writes a reshape.
//!
//! `apply_reshape` is the one place every reshaping stage's wording and span reach the diagnostic,
//! so its properties are those the emitter's design states for every reshape: the class's header
//! replaces rustc's message, the plan's notes and then its helps replace rustc's notes and
//! suggestions (a fix follows the explanation it repairs), and the span policy decides which of
//! rustc's spans stay. Each test builds its diagnostic over one in-memory source file.

use std::cell::Cell;

use cargo_cgp_driver::emitter::{Reshape, SpanPolicy, Verdict, apply_reshape};
use cargo_cgp_error_processing::ClassPlan;
use rustc_errors::{
    Applicability, CodeSuggestion, DiagInner, DiagMessage, Level, MultiSpan, Style, Subdiag,
    Sublevel, SuggestionStyle, Suggestions,
};
use rustc_span::Span;

use crate::driver::source::with_source;

/// The source the spans point into: `first` is bytes 3 to 8 and `second` bytes 17 to 23.
const SOURCE: &str = "fn first() {}\nfn second() {}\n";
const FIRST: (u32, u32) = (3, 8);
const SECOND: (u32, u32) = (17, 23);

/// rustc's diagnostic before a reshape: a header, a labelled primary caret at `primary`, a labelled
/// secondary span at `secondary`, one note of its own, and one suggestion.
fn rustc_diagnostic(primary: Span, secondary: Span) -> DiagInner {
    let mut diag = DiagInner::new(Level::Error, "the name `X` is defined multiple times");
    let mut span = MultiSpan::from_span(primary);
    span.push_span_label(primary, "`X` redefined here");
    span.push_span_label(secondary, "previous definition of `X` here");
    diag.span = span;
    diag.children.push(Subdiag {
        level: Sublevel::Note,
        messages: vec![(DiagMessage::from("a note rustc wrote"), Style::NoStyle)],
        span: MultiSpan::new(),
    });
    diag.suggestions = Suggestions::Enabled(vec![CodeSuggestion {
        substitutions: Vec::new(),
        msg: DiagMessage::from("a suggestion rustc wrote"),
        style: SuggestionStyle::ShowCode,
        applicability: Applicability::MaybeIncorrect,
    }]);
    diag
}

/// A plan with one note and two helps.
fn plan() -> ClassPlan {
    ClassPlan {
        header: "[CGP-E061] the reshaped header".to_owned(),
        notes: vec!["why it fails".to_owned()],
        helps: vec!["the first fix".to_owned(), "the second fix".to_owned()],
    }
}

/// The text of each child, with its level.
fn children(diag: &DiagInner) -> Vec<(Sublevel, String)> {
    diag.children
        .iter()
        .map(|child| {
            let text = child
                .messages
                .iter()
                .map(|(message, _)| match message {
                    DiagMessage::Str(text) => text.to_string(),
                    _ => String::new(),
                })
                .collect();
            (child.level, text)
        })
        .collect()
}

/// Every label of `diag`'s span, as its range and text.
fn labels(diag: &DiagInner) -> Vec<(Span, String)> {
    diag.span
        .span_labels()
        .into_iter()
        .filter_map(|label| match label.label {
            Some(DiagMessage::Str(text)) => Some((label.span, text.to_string())),
            _ => None,
        })
        .collect()
}

// Property: for all diag and plan, after `apply_reshape(diag, plan, policy)` the main message is
//   `plan.header` alone, the children are `plan.notes` as notes followed by `plan.helps` as helps,
//   and rustc's own children and suggestions are gone, whatever the policy.
#[test]
fn the_plan_replaces_the_header_notes_and_suggestions() {
    with_source(SOURCE, |_, span| {
        let mut diag = rustc_diagnostic(span(SECOND), span(FIRST));
        apply_reshape(&mut diag, plan(), SpanPolicy::Keep);
        assert_eq!(diag.messages.len(), 1, "the header is one message");
        assert!(
            matches!(&diag.messages[0].0, DiagMessage::Str(text) if text == "[CGP-E061] the reshaped header"),
            "the header is the plan's",
        );
        assert_eq!(
            children(&diag),
            vec![
                (Sublevel::Note, "why it fails".to_owned()),
                (Sublevel::Help, "the first fix".to_owned()),
                (Sublevel::Help, "the second fix".to_owned()),
            ],
            "the notes come first, then the helps, and rustc's note is gone",
        );
        assert!(
            matches!(&diag.suggestions, Suggestions::Enabled(suggestions) if suggestions.is_empty()),
            "rustc's suggestions are gone",
        );
    });
}

// Property: `SpanPolicy::Keep` leaves rustc's spans and labels exactly as they were.
#[test]
fn keep_leaves_the_spans_and_labels() {
    with_source(SOURCE, |_, span| {
        let mut diag = rustc_diagnostic(span(SECOND), span(FIRST));
        let before = diag.span.clone();
        apply_reshape(&mut diag, plan(), SpanPolicy::Keep);
        assert_eq!(diag.span, before, "the span is unchanged");
    });
}

// Property: `SpanPolicy::Primary` keeps only the primary span, without a label.
#[test]
fn primary_keeps_the_primary_span_alone() {
    with_source(SOURCE, |_, span| {
        let mut diag = rustc_diagnostic(span(SECOND), span(FIRST));
        apply_reshape(&mut diag, plan(), SpanPolicy::Primary);
        assert_eq!(
            diag.span.primary_spans(),
            &[span(SECOND)],
            "one primary span"
        );
        assert!(
            labels(&diag).is_empty(),
            "no label stays: {:?}",
            labels(&diag)
        );
    });
}

// Concern: a reshape of a diagnostic without a primary span could invent one or lose the span.
// Handled: the primary-span policies change the span only when the diagnostic has a primary span.
#[test]
fn primary_leaves_a_diagnostic_without_a_primary_span_alone() {
    with_source(SOURCE, |_, _| {
        for policy in [SpanPolicy::Primary, SpanPolicy::PrimaryCallSite] {
            let mut diag = DiagInner::new(Level::Error, "a message");
            apply_reshape(&mut diag, plan(), policy);
            assert!(
                diag.span.primary_spans().is_empty(),
                "{policy:?} adds no span to a diagnostic without one",
            );
        }
    });
}

// Property: for a span outside any macro expansion, `SpanPolicy::PrimaryCallSite` keeps the same
//   span `SpanPolicy::Primary` does, since such a span is its own call site.
#[test]
fn primary_call_site_of_a_written_span_is_the_span() {
    with_source(SOURCE, |_, span| {
        let mut diag = rustc_diagnostic(span(SECOND), span(FIRST));
        apply_reshape(&mut diag, plan(), SpanPolicy::PrimaryCallSite);
        assert_eq!(
            diag.span.primary_spans(),
            &[span(SECOND)],
            "one primary span"
        );
        assert!(
            labels(&diag).is_empty(),
            "no label stays: {:?}",
            labels(&diag)
        );
    });
}

// Property: `SpanPolicy::TwoDefinitions { first, second }` labels the primary span `second` and the
//   span rustc labelled as the earlier definition `first`, and keeps no other label.
#[test]
fn two_definitions_labels_both_definitions() {
    with_source(SOURCE, |_, span| {
        let mut diag = rustc_diagnostic(span(SECOND), span(FIRST));
        apply_reshape(
            &mut diag,
            plan(),
            SpanPolicy::TwoDefinitions {
                first: "declared first here",
                second: "declared again here",
            },
        );
        let mut found = labels(&diag);
        found.sort_by_key(|(span, _)| span.lo());
        assert_eq!(
            found,
            vec![
                (span(FIRST), "declared first here".to_owned()),
                (span(SECOND), "declared again here".to_owned()),
            ],
            "each definition carries its own label",
        );
        assert_eq!(
            diag.span.primary_spans(),
            &[span(SECOND)],
            "the caret stays"
        );
    });
}

// Property: for all verdicts v other than `Continue`, `v.or_else(next) == v` without calling
//   `next`; `Continue.or_else(next) == next()`. A later stage reads a diagnostic only when every
//   earlier one left it.
#[test]
fn a_decided_verdict_ends_the_chain() {
    let reshape = Verdict::Reshape(Reshape::new(plan(), SpanPolicy::Keep));
    for decided in [Verdict::Drop, reshape] {
        let called = Cell::new(false);
        let verdict = decided.clone().or_else(|| {
            called.set(true);
            Verdict::Continue
        });
        assert_eq!(verdict, decided, "a decided verdict is kept");
        assert!(
            !called.get(),
            "the next stage does not run after {decided:?}"
        );
    }
    assert_eq!(
        Verdict::Continue.or_else(|| Verdict::Drop),
        Verdict::Drop,
        "a diagnostic no stage took goes to the next one",
    );
}
