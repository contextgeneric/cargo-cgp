//! The `DiagInner`-editing helpers the emitter drives.
//!
//! These are the small functions that read from or rewrite a `DiagInner` in place — the seam
//! between the rustc diagnostic type and the rustc-free [error
//! processing](cargo_cgp_error_processing) that decides *what* the text should say. Keeping them
//! here leaves [`CgpEmitter`](super::CgpEmitter) with only the orchestration.

use std::borrow::Cow;

use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::{
    AmbiguousNames, COALESCED_SITE_LABEL, DiagKind, Origin, bound_candidate_list,
    context_has_hasfield_impls, is_method_probe_advice_text, is_question_mark_cascade_text,
    mentions_wiring_text, postprocess_fragments, postprocess_message,
};
use rustc_errors::codes::{E0271, E0599};
use rustc_errors::{
    CodeSuggestion, DiagInner, DiagMessage, MultiSpan, Style, Subdiag, Sublevel, Suggestions,
};
use rustc_span::Span;

/// The span of a block that coalesces `members`, ordered by source position: the first member's
/// caret is the block's one primary span, and each other member's caret is a secondary span labelled
/// [`COALESCED_SITE_LABEL`], so an editor shows the block as one diagnostic with related locations
/// rather than one diagnostic per caret. A site repeated across members is labelled once.
pub(crate) fn merged_span(members: &[&DiagInner]) -> MultiSpan {
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
pub(crate) fn two_definitions_span(span: &MultiSpan, first: &str, second: &str) -> MultiSpan {
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
        Some(E0271) => DiagKind::TypeMismatch,
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
pub(crate) fn subdiag(level: Sublevel, message: String) -> Subdiag {
    Subdiag {
        level,
        messages: vec![(DiagMessage::Str(message.into()), Style::NoStyle)],
        span: MultiSpan::new(),
    }
}

/// A span-independent signature of a diagnostic's rendered text — its main message plus each
/// child's message, in order. Used to de-duplicate a text-rewritten CGP diagnostic the typed
/// resolver *declined*: the same rewritten wiring error re-reported at another span renders the
/// same text, so the emitter shows the first and suppresses the rest. It deliberately ignores
/// spans and labels, since those are exactly what differ between the duplicates.
pub(crate) fn message_signature(diag: &DiagInner) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for (message, _) in &diag.messages {
        if let DiagMessage::Str(text) = message {
            parts.push(text);
        }
    }
    for child in &diag.children {
        for (message, _) in &child.messages {
            if let DiagMessage::Str(text) = message {
                parts.push(text);
            }
        }
    }
    parts.join("\u{1f}")
}

/// Whether `diag` is a downstream `?`-operator cascade — the `Try` / `FromResidual` error rustc
/// emits when the `?` in `expr?` is applied to a value whose type it could not resolve because an
/// earlier trait bound on that same expression failed
/// ([`is_question_mark_cascade_text`]). On its own this is not enough to suppress
/// — a genuine `?` misuse reads the same — so the emitter pairs it with a span check: the cascade is
/// dropped only when it sits on an expression where a CGP wiring failure was already reported, where
/// it merely restates that failure in `Try` terms while dumping the unresolved projected type.
pub(crate) fn is_question_mark_cascade(diag: &DiagInner) -> bool {
    main_message_text(diag).is_some_and(is_question_mark_cascade_text)
}

/// Whether `diag` is the method probe's "no method named `…` found" `E0599` (or "no associated
/// function or constant named `…`" for a type-relative path call), the probe having found no impl
/// of a trait that declares the item. Its main message is a translatable one, so it is recognized by the arguments
/// that message names (`item_kind`, `item_ident`), which the resolution-class `E0599`s ("no variant
/// named …") do not carry. The probe builds the method-bounds `E0599` from the same diagnostic and
/// replaces its message with a plain string, so a plain-string message rules this shape out.
pub(crate) fn is_no_matching_item(diag: &DiagInner) -> bool {
    diag.code == Some(E0599)
        && main_message_text(diag).is_none()
        && diag.args.contains_key("item_kind")
        && diag.args.contains_key("item_ident")
}

/// Strip rustc's method-probe advice from a method-bounds `E0599` the resolver declined: the
/// "this is an associated function, not a method" caret label, the "found the following
/// associated functions …" note with its "the candidate is defined in …" follow-up, and the
/// "use associated function syntax instead" structured suggestion. All are artifacts of CGP's
/// `self`-less provider methods ([`is_method_probe_advice_text`]): the probe blames the call
/// syntax, and the suggestion it offers is wrong, while the unmet wiring bound the diagnostic
/// also names is the real cause, which this leaves in place (with the notes gone, it is the first
/// thing after the caret). rustc's other suggestions stay, such as the bound to restrict a type
/// parameter with.
pub(crate) fn strip_method_probe_advice(diag: &mut DiagInner) {
    retain_suggestions(
        diag,
        |suggestion| !matches!(&suggestion.msg, DiagMessage::Str(text) if is_method_probe_advice_text(text)),
    );
    diag.children.retain(|child| {
        !child.messages.iter().any(|(message, _)| {
            matches!(message, DiagMessage::Str(text) if is_method_probe_advice_text(text))
        })
    });
    strip_multispan_labels(&mut diag.span, is_method_probe_advice_text);
}

/// Keep only the structured suggestions of `diag` that `keep` accepts, whichever state they are in.
pub(crate) fn retain_suggestions(diag: &mut DiagInner, keep: impl Fn(&CodeSuggestion) -> bool) {
    match &mut diag.suggestions {
        Suggestions::Enabled(suggestions) => suggestions.retain(|suggestion| keep(suggestion)),
        Suggestions::Sealed(suggestions) => {
            *suggestions = suggestions
                .iter()
                .filter(|suggestion| keep(suggestion))
                .cloned()
                .collect();
        }
        Suggestions::Disabled => {}
    }
}

/// Cut each candidate list among `diag`'s sub-messages back to the length rustc prints outside a
/// verbose session ([`bound_candidate_list`]), undoing the one list the driver's injected
/// `--verbose` un-elides.
pub(crate) fn bound_candidate_lists(diag: &mut DiagInner) {
    for child in &mut diag.children {
        for (message, _) in child.messages.iter_mut() {
            if let DiagMessage::Str(text) = message
                && let Some(bounded) = bound_candidate_list(text)
            {
                *message = DiagMessage::Str(Cow::Owned(bounded));
            }
        }
    }
}

/// Drop the labels of a [`MultiSpan`] whose text matches `drop`, keeping the spans themselves
/// (an unlabelled primary span still renders its caret). The span is rebuilt only when a label
/// actually matches.
fn strip_multispan_labels(span: &mut MultiSpan, drop: fn(&str) -> bool) {
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
fn rebuilt_multispan(span: &MultiSpan, labels: Vec<(Span, DiagMessage)>) -> MultiSpan {
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

/// Whether any of `diag`'s messages, its header or a child's, mentions a CGP wiring trait
/// ([`mentions_wiring_text`]): the evidence that a method-bounds `E0599` the resolver declined is a
/// CGP failure, so its method-probe advice is stripped.
pub(crate) fn mentions_wiring(diag: &DiagInner) -> bool {
    fn any(messages: &[(DiagMessage, Style)]) -> bool {
        messages.iter().any(|(message, _)| match message {
            DiagMessage::Str(text) => mentions_wiring_text(text),
            _ => false,
        })
    }
    any(&diag.messages) || diag.children.iter().any(|child| any(&child.messages))
}

/// Whether any plain-string message or span label across `diag` and its children shows the
/// context implementing `HasField` for a field: the whole-diagnostic fact the missing-field rewrite
/// needs before it calls a field missing.
pub(crate) fn mentions_hasfield_impls(diag: &DiagInner) -> bool {
    // A message's styled fragments are read joined: rustc highlights the `is` of `is implemented
    // for it` apart, so no single fragment carries the landmark.
    fn in_messages<S>(messages: &[(DiagMessage, S)]) -> bool {
        let joined: String = messages
            .iter()
            .filter_map(|(message, _)| match message {
                DiagMessage::Str(text) => Some(text.as_ref()),
                _ => None,
            })
            .collect();
        context_has_hasfield_impls(&joined)
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
pub(crate) fn postprocess_messages(
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
