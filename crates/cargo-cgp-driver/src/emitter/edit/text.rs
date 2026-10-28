//! The rustc-free text transforms, applied to a diagnostic's messages and labels in place.

use std::borrow::Cow;

use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::{
    AmbiguousNames, Origin, postprocess_fragments, postprocess_message,
};
use rustc_errors::{DiagInner, DiagMessage, MultiSpan, Style};
use rustc_span::Span;

use crate::emitter::edit::{mentions_hasfield_impls, rebuilt_multispan};

/// Rewrite each plain-string message in place through `rewrite`, leaving its style and any
/// Fluent message untouched. The rewrite function consults the name map only for a message that
/// parses as a CGP wiring form.
pub fn rewrite_messages<S>(
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

/// Post-process a message in place through [`postprocess_message`], leaving any Fluent message
/// untouched.
///
/// One rustc "message" can be several *styled fragments* — `Diag::highlighted_*` stores one entry
/// per [`StringPart`](rustc_errors::StringPart), and the renderer concatenates them. Each fragment
/// is post-processed on its own first, which keeps rustc's highlighting; but a construct split
/// across fragments is invisible that way, and rustc splits precisely where it highlights a
/// *difference* between two types — so its "similar impl" hint shreds a `Symbol<3, Chars<'B', …>>`
/// into a fragment per character and no fragment matches. So the fragments are then read as the one
/// line they render as ([`postprocess_fragments`]), and when that recovers something the per-fragment
/// pass could not, the whole message collapses to that single unstyled string. Losing the
/// highlighting is the price of the recovery, and it is only paid when there was a construct to
/// recover.
pub fn postprocess_messages(
    messages: &mut Vec<(DiagMessage, Style)>,
    has_field_impls: bool,
    origin: Origin,
    ambiguous: &AmbiguousNames,
) {
    for (message, _) in messages.iter_mut() {
        if let DiagMessage::Str(text) = message
            && let Some(rewritten) = postprocess_message(text, has_field_impls, origin, ambiguous)
        {
            *message = DiagMessage::Str(Cow::Owned(rewritten));
        }
    }

    // Only a wholly plain-string message can be read as one line; a Fluent fragment has no text
    // here to join, so such a message keeps its per-fragment result.
    let fragments: Vec<&str> = messages
        .iter()
        .filter_map(|(message, _)| match message {
            DiagMessage::Str(text) => Some(text.as_ref()),
            _ => None,
        })
        .collect();
    if fragments.len() != messages.len() {
        return;
    }

    if let Some(joined) = postprocess_fragments(&fragments, has_field_impls, origin, ambiguous) {
        *messages = vec![(DiagMessage::Str(Cow::Owned(joined)), Style::NoStyle)];
    }
}

/// Post-process each of a [`MultiSpan`]'s labels — the caret and secondary-label text the
/// emitter renders alongside the source. The span is rebuilt only when a label actually
/// changes, so an unaffected diagnostic keeps its exact `MultiSpan`.
pub fn postprocess_multispan(
    span: &mut MultiSpan,
    has_field_impls: bool,
    origin: Origin,
    ambiguous: &AmbiguousNames,
) {
    let labels: Vec<(Span, DiagMessage)> = span.span_labels_raw().to_vec();
    if labels.is_empty() {
        return;
    }

    let mut changed = false;
    let new_labels: Vec<(Span, DiagMessage)> = labels
        .into_iter()
        .map(|(span, message)| {
            if let DiagMessage::Str(text) = &message
                && let Some(rewritten) =
                    postprocess_message(text, has_field_impls, origin, ambiguous)
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
    *span = rebuilt_multispan(span, new_labels);
}

/// Post-process every message and span label of `diag` and its children: the final cleanup pass
/// after a transform. It strips module paths, resugars `Symbol!`, `Path!`, and the type-level lists,
/// and rewords an unmet `HasField` bound. Whether the context implements `HasField` for any field
/// is a fact of the whole diagnostic (the "similar impl" landmark can sit far from the clause), so
/// it is decided once up front and passed into each per-message rewrite. `origin` says whether the
/// tool rewrote the diagnostic (see [`Origin`]).
pub fn postprocess_diag(diag: &mut DiagInner, origin: Origin, ambiguous: &AmbiguousNames) {
    let has_field_impls = mentions_hasfield_impls(diag);
    postprocess_messages(&mut diag.messages, has_field_impls, origin, ambiguous);
    postprocess_multispan(&mut diag.span, has_field_impls, origin, ambiguous);
    for child in &mut diag.children {
        postprocess_messages(&mut child.messages, has_field_impls, origin, ambiguous);
        postprocess_multispan(&mut child.span, has_field_impls, origin, ambiguous);
    }
}
