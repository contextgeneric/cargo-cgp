//! Applying a text transformation across a diagnostic's human-readable fields.

use cargo_metadata::diagnostic::Diagnostic;

/// Apply `transform` to the diagnostic's `message` and `rendered` text, replacing each
/// with the transformed form and returning whether any of them changed.
///
/// `transform` returns the rewritten text and whether it differs from its input, so a
/// preprocessor can both edit the diagnostic and report — through the returned flag —
/// that it recognized something. Only `message` and `rendered` are touched: `rendered` is
/// the text the tool actually prints, and it already contains the fully-rendered form of
/// the whole diagnostic (spans, notes, children), so rewriting it is what changes the
/// output. Nested structured fields are left alone until structured JSON output needs
/// them.
pub(crate) fn map_diagnostic_text(
    diagnostic: &mut Diagnostic,
    transform: impl Fn(&str) -> (String, bool),
) -> bool {
    let mut changed = false;

    let (message, message_changed) = transform(&diagnostic.message);
    if message_changed {
        diagnostic.message = message;
        changed = true;
    }

    if let Some(rendered) = diagnostic.rendered.take() {
        let (new_rendered, rendered_changed) = transform(&rendered);
        changed |= rendered_changed;
        diagnostic.rendered = Some(new_rendered);
    }

    changed
}
