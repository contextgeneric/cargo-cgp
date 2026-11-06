//! Readers and predicates over a diagnostic's messages, code, and arguments.

use cargo_cgp_error_processing::{
    DiagKind, context_has_hasfield_impls, is_question_mark_cascade_text, mentions_wiring_text,
};
use rustc_errors::codes::{E0271, E0599};
use rustc_errors::{DiagInner, DiagMessage, MultiSpan, Style};

/// The text of the diagnostic's main message, when it is a plain string.
pub fn main_message_text(diag: &DiagInner) -> Option<&str> {
    match diag.messages.first() {
        Some((DiagMessage::Str(text), _)) => Some(text),
        _ => None,
    }
}

/// Map a diagnostic's `rustc` error code to the rustc-free [`DiagKind`] the diagnosis planner
/// keys on: `E0271` is a field-type mismatch, `E0599` a consumer-method call, and everything else
/// (chiefly `E0277`) a plain check/bound failure.
pub fn diag_kind(diag: &DiagInner) -> DiagKind {
    match diag.code {
        Some(E0271) => DiagKind::TypeMismatch,
        Some(E0599) => DiagKind::MethodNotFound,
        _ => DiagKind::Check,
    }
}

/// A span-independent signature of a diagnostic's rendered text: its main message plus each child's
/// message, in order. Used to de-duplicate a text-rewritten CGP diagnostic the typed resolver
/// *declined*: the same rewritten wiring error re-reported at another span renders the same text,
/// so the emitter shows the first and suppresses the rest. It deliberately ignores spans and
/// labels, since the duplicates differ in exactly those.
pub fn message_signature(diag: &DiagInner) -> String {
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

/// Whether `diag` is a downstream `?`-operator cascade: the `Try` / `FromResidual` error rustc
/// emits when the `?` in `expr?` is applied to a value whose type it could not resolve because an
/// earlier trait bound on that same expression failed ([`is_question_mark_cascade_text`]). On its
/// own this is not enough to suppress (a genuine `?` misuse reads the same), so the emitter pairs
/// it with a span check: the cascade is dropped only when it sits on an expression where a CGP
/// wiring failure was already reported, where it merely restates that failure in `Try` terms while
/// dumping the unresolved projected type.
pub fn is_question_mark_cascade(diag: &DiagInner) -> bool {
    main_message_text(diag).is_some_and(is_question_mark_cascade_text)
}

/// Whether `diag` is the method probe's "no method named `…` found" `E0599` (or "no associated
/// function or constant named `…`" for a type-relative path call), the probe having found no impl
/// of a trait that declares the item. Its main message is a translatable one, so it is recognized
/// by the arguments that message names (`item_kind`, `item_ident`), which the resolution-class
/// `E0599`s ("no variant named …") do not carry. The probe builds the method-bounds `E0599` from
/// the same diagnostic and replaces its message with a plain string, so a plain-string message
/// rules this shape out.
pub fn is_no_matching_item(diag: &DiagInner) -> bool {
    diag.code == Some(E0599)
        && main_message_text(diag).is_none()
        && diag.args.contains_key("item_kind")
        && diag.args.contains_key("item_ident")
}

/// Whether any of `diag`'s messages, its header or a child's, mentions a CGP wiring trait
/// ([`mentions_wiring_text`]): the evidence that a method-bounds `E0599` the resolver declined is a
/// CGP failure, so its method-probe advice is stripped.
pub fn mentions_wiring(diag: &DiagInner) -> bool {
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
pub fn mentions_hasfield_impls(diag: &DiagInner) -> bool {
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

/// Whether any plain-string message of `diag`, its main message, a child's, or a span label,
/// satisfies `pred`. Used to recognize a diagnostic whose *main* message is a Fluent (non-`Str`)
/// message but whose help or label carries the signal as plain text.
pub fn diag_mentions(diag: &DiagInner, pred: fn(&str) -> bool) -> bool {
    let in_messages = |messages: &[(DiagMessage, Style)]| {
        messages
            .iter()
            .any(|(message, _)| matches!(message, DiagMessage::Str(text) if pred(text)))
    };
    let in_labels = |span: &MultiSpan| {
        span.span_labels()
            .into_iter()
            .any(|label| matches!(label.label, Some(DiagMessage::Str(text)) if pred(&text)))
    };
    in_messages(&diag.messages)
        || in_labels(&diag.span)
        || diag
            .children
            .iter()
            .any(|child| in_messages(&child.messages) || in_labels(&child.span))
}
