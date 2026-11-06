//! Edits of a diagnostic's header, notes, helps, and suggestions.

use std::borrow::Cow;

use cargo_cgp_error_processing::{bound_candidate_list, is_method_probe_advice_text};
use rustc_errors::{
    CodeSuggestion, DiagInner, DiagMessage, MultiSpan, Style, Subdiag, Sublevel, Substitution,
    Suggestions,
};

use crate::emitter::edit::strip_multispan_labels;

/// Replace a diagnostic's main message with `header`, keeping its spans and their labels while
/// dropping its children (rustc's redundant coherence notes, such as "downstream crates may
/// implement …") and its structured suggestions. For a duplicate-key conflict, the kept carets are
/// the two that point at the colliding entries, "first implementation here" and "conflicting
/// implementation". The header says *what* collided; the kept carets say *where*.
pub fn replace_header(diag: &mut DiagInner, header: String) {
    diag.messages = vec![(DiagMessage::Str(header.into()), Style::NoStyle)];
    diag.children = Vec::new();
    diag.suggestions = Suggestions::Enabled(Vec::new());
}

/// Build a plain-text sub-diagnostic (a `help` or `note`) with no span: the shape every
/// diagnosis note and derive help takes.
pub fn subdiag(level: Sublevel, message: String) -> Subdiag {
    Subdiag {
        level,
        messages: vec![(DiagMessage::Str(message.into()), Style::NoStyle)],
        span: MultiSpan::new(),
    }
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
pub fn strip_method_probe_advice(diag: &mut DiagInner) {
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
pub fn retain_suggestions(diag: &mut DiagInner, keep: impl Fn(&CodeSuggestion) -> bool) {
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

/// Keep only the substitutions of `diag`'s structured suggestions that `keep` accepts, whichever
/// state the suggestions are in, and drop a suggestion left with none. rustc offers several
/// alternatives as one suggestion's substitutions (each trait that would bound a parameter), so a
/// wrong alternative goes without the right ones beside it.
pub fn retain_substitutions(diag: &mut DiagInner, keep: impl Fn(&Substitution) -> bool) {
    let filter = |mut suggestion: CodeSuggestion| {
        suggestion
            .substitutions
            .retain(|substitution| keep(substitution));
        (!suggestion.substitutions.is_empty()).then_some(suggestion)
    };
    match &mut diag.suggestions {
        Suggestions::Enabled(suggestions) => {
            *suggestions = std::mem::take(suggestions)
                .into_iter()
                .filter_map(filter)
                .collect();
        }
        Suggestions::Sealed(suggestions) => {
            *suggestions = suggestions.iter().cloned().filter_map(filter).collect();
        }
        Suggestions::Disabled => {}
    }
}

/// Cut each candidate list among `diag`'s sub-messages back to the length rustc prints outside a
/// verbose session ([`bound_candidate_list`]), undoing the one list the driver's injected
/// `--verbose` un-elides.
pub fn bound_candidate_lists(diag: &mut DiagInner) {
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
