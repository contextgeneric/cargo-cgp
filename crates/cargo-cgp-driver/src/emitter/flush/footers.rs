//! Rebuilding the "detailed explanations" footer to name only the codes still shown.

use std::collections::HashSet;

use cargo_cgp_error_processing::{
    explain_footer_codes, explain_list_text, explain_pointer_text, is_explain_footer_text,
    is_explain_list_text,
};
use rustc_errors::{DiagInner, DiagMessage, Level, Style};

use crate::emitter::edit::main_message_text;

/// Rewrite the trailing "detailed explanations" footer of a flushed diagnostic list so it names
/// `rustc --explain` codes only for errors still shown.
///
/// rustc builds that footer in `print_error_count` from every error code it *registered* as
/// diagnostics arrived, which is a superset of what this emitter emits: it suppresses a
/// re-reported failure, a `?`-operator cascade, and a `#[cgp_impl]` mistake's sibling burst, and
/// it *merges* the failures sharing a root cause into one block whose code is the first member's.
/// Left alone the footer contradicts the output, offering `--explain` for a code no error carries.
///
/// It runs over the flushed list rather than on the footer's arrival because only here is the
/// surviving set final: coalescing happens at flush, after every diagnostic has been buffered.
/// The kept codes are the footer's own codes filtered by what survived, never a fresh set built
/// from the survivors: rustc lists only codes that *have* an extended explanation, and
/// intersecting preserves that filtering instead of second-guessing it. A footer whose codes all
/// survive is rewritten to the identical text, so output the emitter did not touch is unaffected.
pub fn rebuild_explain_footers(to_emit: &mut Vec<DiagInner>) {
    let is_footer = |diag: &DiagInner| {
        diag.level() == Level::FailureNote
            && main_message_text(diag).is_some_and(is_explain_footer_text)
    };
    if !to_emit.iter().any(is_footer) {
        return;
    }

    // The rust codes still shown, and the explainable ones the footer itself names (the list line
    // carries every one, the pointer line only the first, so their union is the whole set).
    let surviving: HashSet<String> = to_emit
        .iter()
        .filter(|diag| diag.level() != Level::FailureNote)
        .filter_map(|diag| diag.code)
        .map(|code| code.to_string())
        .collect();
    let mut explainable: Vec<String> = Vec::new();
    for diag in to_emit.iter().filter(|diag| is_footer(diag)) {
        for code in explain_footer_codes(main_message_text(diag).unwrap_or_default()) {
            if !explainable.contains(&code) {
                explainable.push(code);
            }
        }
    }
    // A footer line always names at least one code, so parsing none from any of them means the
    // parse failed, meaning that rustc reworded, not that the line names nothing. Rebuilding on that would
    // delete the footer outright, so leave it exactly as rustc wrote it: a stale footer is a far
    // smaller fault than a silently missing one, and it is the pre-rebuild behaviour.
    if explainable.is_empty() {
        return;
    }
    let kept: Vec<String> = explainable
        .into_iter()
        .filter(|code| surviving.contains(code))
        .collect();

    let rebuilt: Vec<DiagInner> = to_emit
        .drain(..)
        .filter_map(|diag| {
            if !is_footer(&diag) {
                return Some(diag);
            }
            let text = main_message_text(&diag).unwrap_or_default();
            let Some(first) = kept.first() else {
                // Every explainable code was suppressed, so there is nothing left to point at
                // and both footer lines go.
                return None;
            };
            if is_explain_list_text(text) {
                // The list line is only right for two or more codes; with one, the pointer line
                // below carries it alone.
                if kept.len() < 2 {
                    return None;
                }
                return Some(footer_note(&diag, &explain_list_text(&kept)));
            }
            Some(footer_note(
                &diag,
                &explain_pointer_text(first, kept.len() == 1),
            ))
        })
        .collect();
    *to_emit = rebuilt;
}

/// A rebuilt "detailed explanations" footer note: `template` cloned from the original footer
/// diagnostic (keeping its `FailureNote` level and empty span) with `message` as its only text.
fn footer_note(template: &DiagInner, message: &str) -> DiagInner {
    let mut note = template.clone();
    note.messages = vec![(DiagMessage::Str(message.to_string().into()), Style::NoStyle)];
    note
}
