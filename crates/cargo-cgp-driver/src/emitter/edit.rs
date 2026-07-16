//! The `DiagInner`-editing helpers the emitter drives.
//!
//! These are the small functions that read from or rewrite a `DiagInner` in place — the seam
//! between the rustc diagnostic type and the rustc-free [error
//! processing](cargo_cgp_error_processing) that decides *what* the text should say. Keeping them
//! here leaves [`CgpEmitter`](super::CgpEmitter) with only the orchestration.

use std::borrow::Cow;

use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::{DiagKind, context_has_hasfield_impls, postprocess_message};
use rustc_errors::codes::{E0271, E0599};
use rustc_errors::{DiagInner, DiagMessage, Level, MultiSpan, Style, Subdiag, Suggestions};
use rustc_span::Span;

/// The text of the diagnostic's main message, when it is a plain string.
pub(crate) fn main_message_text(diag: &DiagInner) -> Option<&str> {
    match diag.messages.first() {
        Some((DiagMessage::Str(text), _)) => Some(text),
        _ => None,
    }
}

/// Map a diagnostic's `rustc` error code to the rustc-free [`DiagKind`] the diagnosis planner
/// keys on: `E0271` is a field-type mismatch, `E0599` a consumer-method call, and everything else
/// (chiefly `E0277`) a plain check/bound failure.
pub(crate) fn diag_kind(diag: &DiagInner) -> DiagKind {
    match diag.code {
        Some(E0271) => DiagKind::FieldMismatch,
        Some(E0599) => DiagKind::MethodNotFound,
        _ => DiagKind::Check,
    }
}

/// Replace a diagnostic's main message with `header`, keeping its spans and their labels — for a
/// duplicate-key conflict, the two carets ("first implementation here" and "conflicting
/// implementation") that point at the colliding entries — while dropping its children (rustc's
/// redundant coherence notes, such as "downstream crates may implement …") and its structured
/// suggestions. The header says *what* collided; the kept carets say *where*.
pub(crate) fn replace_header(diag: &mut DiagInner, header: String) {
    diag.messages = vec![(DiagMessage::Str(header.into()), Style::NoStyle)];
    diag.children = Vec::new();
    diag.suggestions = Suggestions::Enabled(Vec::new());
}

/// Build a plain-text sub-diagnostic (a `help` or `note`) with no span — the shape every
/// diagnosis note and derive help takes.
pub(crate) fn subdiag(level: Level, message: String) -> Subdiag {
    Subdiag {
        level,
        messages: vec![(DiagMessage::Str(message.into()), Style::NoStyle)],
        span: MultiSpan::new(),
    }
}

/// Every span a diagnostic carries — its primary and labelled spans plus each child's — the pool
/// the use-site resolver searches for one that lands on the failing context's type definition.
pub(crate) fn diagnostic_spans(diag: &DiagInner) -> Vec<Span> {
    let mut spans: Vec<Span> = diag.span.primary_spans().to_vec();
    spans.extend(diag.span.span_labels().into_iter().map(|label| label.span));
    for child in &diag.children {
        spans.extend(child.span.primary_spans());
        spans.extend(child.span.span_labels().into_iter().map(|label| label.span));
    }
    spans
}

/// Whether any of `diag`'s messages — its header or a child's — mentions a CGP wiring trait. This
/// is the cheap pre-filter that decides whether to attempt the (expensive) typed resolution at all,
/// so that any wiring diagnostic is considered.
pub(crate) fn mentions_wiring(diag: &DiagInner) -> bool {
    fn any(messages: &[(DiagMessage, Style)]) -> bool {
        messages.iter().any(|(message, _)| match message {
            // `HasField` catches a use-site failure (a consumer-method `E0599`), whose text names
            // the missing leaf but not `CanUseComponent`/`IsProviderFor`.
            DiagMessage::Str(text) => {
                text.contains("CanUseComponent")
                    || text.contains("IsProviderFor")
                    || text.contains("HasField")
            }
            _ => false,
        })
    }
    any(&diag.messages) || diag.children.iter().any(|child| any(&child.messages))
}

/// Whether any plain-string message or span label across `diag` and its children shows the
/// context implementing `HasField` for a field — the whole-diagnostic fact the missing-field
/// rewrite needs to tell a single missing field from a missing `#[derive(HasField)]`.
pub(crate) fn mentions_hasfield_impls(diag: &DiagInner) -> bool {
    fn in_messages<S>(messages: &[(DiagMessage, S)]) -> bool {
        messages.iter().any(|(message, _)| match message {
            DiagMessage::Str(text) => context_has_hasfield_impls(text),
            _ => false,
        })
    }
    fn in_span(span: &MultiSpan) -> bool {
        span.span_labels_raw()
            .iter()
            .any(|(_, message)| match message {
                DiagMessage::Str(text) => context_has_hasfield_impls(text),
                _ => false,
            })
    }
    in_messages(&diag.messages)
        || in_span(&diag.span)
        || diag
            .children
            .iter()
            .any(|child| in_messages(&child.messages) || in_span(&child.span))
}

/// Rewrite each plain-string message in place through `rewrite`, leaving its style and any
/// Fluent message untouched. The rewrite function consults the name map only for a message that
/// parses as a CGP wiring form.
pub(crate) fn rewrite_messages<S>(
    messages: &mut [(DiagMessage, S)],
    names: &ComponentNameMap,
    rewrite: fn(&str, &ComponentNameMap) -> Option<String>,
) -> bool {
    let mut changed = false;
    for (message, _) in messages.iter_mut() {
        if let DiagMessage::Str(text) = message
            && let Some(rewritten) = rewrite(text, names)
        {
            *message = DiagMessage::Str(Cow::Owned(rewritten));
            changed = true;
        }
    }
    changed
}

/// Post-process each plain-string message in place through [`postprocess_message`], leaving
/// its style and any Fluent message untouched.
pub(crate) fn postprocess_messages<S>(
    messages: &mut [(DiagMessage, S)],
    has_field_impls: bool,
    bare_paths: bool,
) {
    for (message, _) in messages.iter_mut() {
        if let DiagMessage::Str(text) = message
            && let Some(rewritten) = postprocess_message(text, has_field_impls, bare_paths)
        {
            *message = DiagMessage::Str(Cow::Owned(rewritten));
        }
    }
}

/// Post-process each of a [`MultiSpan`]'s labels — the caret and secondary-label text the
/// emitter renders alongside the source. The span is rebuilt only when a label actually
/// changes, so an unaffected diagnostic keeps its exact `MultiSpan`; the primary spans are
/// re-pushed in order (rather than through `from_spans`, which sorts them) so the rendering
/// order is preserved.
pub(crate) fn postprocess_multispan(span: &mut MultiSpan, has_field_impls: bool, bare_paths: bool) {
    let labels: Vec<(Span, DiagMessage)> = span.span_labels_raw().to_vec();
    if labels.is_empty() {
        return;
    }

    let mut changed = false;
    let new_labels: Vec<(Span, DiagMessage)> = labels
        .into_iter()
        .map(|(span, message)| {
            if let DiagMessage::Str(text) = &message
                && let Some(rewritten) = postprocess_message(text, has_field_impls, bare_paths)
            {
                changed = true;
                (span, DiagMessage::Str(Cow::Owned(rewritten)))
            } else {
                (span, message)
            }
        })
        .collect();
    if !changed {
        return;
    }

    let mut rebuilt = MultiSpan::new();
    for primary in span.primary_spans() {
        rebuilt.push_primary_span(*primary);
    }
    for (span, message) in new_labels {
        rebuilt.push_span_diag(span, message);
    }
    *span = rebuilt;
}
