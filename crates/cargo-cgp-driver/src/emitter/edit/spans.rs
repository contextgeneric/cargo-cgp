//! Span edits: the carets and labels a reshaped or merged diagnostic shows.

use cargo_cgp_error_processing::COALESCED_SITE_LABEL;
use rustc_errors::{DiagInner, DiagMessage, MultiSpan};
use rustc_span::Span;

/// The span of a block that coalesces `members`, ordered by source position: the first member's
/// caret is the block's one primary span, and each other member's caret is a secondary span labelled
/// [`COALESCED_SITE_LABEL`], so an editor shows the block as one diagnostic with related locations
/// rather than one diagnostic per caret. A site repeated across members is labelled once.
pub fn merged_span(members: &[&DiagInner]) -> MultiSpan {
    let mut sites = members
        .iter()
        .filter_map(|member| member.span.primary_span());
    let Some(first) = sites.next() else {
        return MultiSpan::new();
    };
    let mut span = MultiSpan::from_span(first);
    let mut labelled = vec![first];
    for site in sites {
        if !labelled.contains(&site) {
            labelled.push(site);
            span.push_span_label(site, COALESCED_SITE_LABEL);
        }
    }
    span
}

/// The span of an `E0428` on a generated name defined twice, relabelled to name what the program
/// wrote: the reported caret takes `second`, and the span rustc labels as the previous definition
/// takes `first`. rustc's own labels name the generated item.
pub fn two_definitions_span(span: &MultiSpan, first: &str, second: &str) -> MultiSpan {
    let Some(primary) = span.primary_span() else {
        return span.clone();
    };
    let mut relabelled = MultiSpan::from_span(primary);
    relabelled.push_span_label(primary, second.to_owned());
    for label in span.span_labels() {
        if !label.is_primary {
            relabelled.push_span_label(label.span, first.to_owned());
        }
    }
    relabelled
}

/// Drop the labels of a [`MultiSpan`] whose text matches `drop`, keeping the spans themselves
/// (an unlabelled primary span still renders its caret). The span is rebuilt only when a label
/// actually matches.
pub fn strip_multispan_labels(span: &mut MultiSpan, drop: fn(&str) -> bool) {
    let labels: Vec<(Span, DiagMessage)> = span.span_labels_raw().to_vec();
    let kept: Vec<(Span, DiagMessage)> = labels
        .iter()
        .filter(|(_, message)| !matches!(message, DiagMessage::Str(text) if drop(text)))
        .cloned()
        .collect();
    if kept.len() == labels.len() {
        return;
    }

    *span = rebuilt_multispan(span, kept);
}

/// `span` with its labels replaced by `labels`, keeping its primary spans in order (rather than
/// through `from_spans`, which sorts them) and its context spans, the unlabelled lines rustc shows
/// around a label for orientation.
pub fn rebuilt_multispan(span: &MultiSpan, labels: Vec<(Span, DiagMessage)>) -> MultiSpan {
    let mut rebuilt = MultiSpan::new();
    for primary in span.primary_spans() {
        rebuilt.push_primary_span(*primary);
    }
    for (label_span, message) in labels {
        rebuilt.push_span_diag(label_span, message);
    }
    for context in span.span_context() {
        rebuilt.push_span_context(*context);
    }
    rebuilt
}

/// Every span a diagnostic carries (its primary and labelled spans plus each child's): the pool
/// the use-site resolver searches for one that lands on the failing context's type definition.
pub fn diagnostic_spans(diag: &DiagInner) -> Vec<Span> {
    let mut spans: Vec<Span> = diag.span.primary_spans().to_vec();
    spans.extend(diag.span.span_labels().into_iter().map(|label| label.span));
    for child in &diag.children {
        spans.extend(child.span.primary_spans());
        spans.extend(child.span.span_labels().into_iter().map(|label| label.span));
    }
    spans
}
