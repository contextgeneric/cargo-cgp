//! The post-processing chain the driver applies to a diagnostic's messages.

use std::borrow::Cow;

use crate::postprocess::ambiguous::AmbiguousNames;
use crate::postprocess::missing_field::rewrite_missing_fields;
use crate::postprocess::resugar_list::resugar_lists;
use crate::postprocess::resugar_path::resugar_path;
use crate::postprocess::resugar_symbol::resugar_symbol;
use crate::postprocess::strip_modules::strip_unambiguous_paths;

/// Who built the message the chain cleans up, which decides how much of the chain applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// The tool built or rewrote the diagnostic, so every stage applies and a path shows as a bare
    /// `@…`, the way the tool's own sentences name one.
    Rewritten,
    /// rustc built the diagnostic and the tool left it alone. The chain then changes a message only
    /// where it recovers a CGP encoding (a `Symbol!`, a path, a type-level list, or an unmet
    /// `HasField` bound), so a plain Rust message stays exactly as rustc printed it; a path shows
    /// in its `Path!(@…)` form, since the reader meets it as a type.
    Untransformed,
}

/// Run the post-processing chain over one message string, returning the rewritten text when the
/// chain changed it and `None` otherwise. For an [`Origin::Untransformed`] message the strips alone
/// never count as a change.
///
/// `ambiguous` holds the type names that name more than one item in the build, whose paths keep
/// their qualifier. `has_field_impls` is the whole-diagnostic fact the field rewrite needs, which
/// the caller computes once with [`context_has_hasfield_impls`](super::context_has_hasfield_impls).
///
/// See `cgp-knowledge-base/cargo-cgp/implementation/error-processing.md` for the stages and why
/// their order matters.
pub fn postprocess_message(
    text: &str,
    has_field_impls: bool,
    origin: Origin,
    ambiguous: &AmbiguousNames,
) -> Option<String> {
    let mut current: Cow<'_, str> = Cow::Borrowed(text);
    // Strip module qualifiers first, so the resugaring stages below match the bare
    // `Symbol`/`Chars`/`PathCons` names, keeping only those that tell two items apart.
    if let Some(stripped) = strip_unambiguous_paths(&current, ambiguous) {
        current = Cow::Owned(stripped);
    }

    let mut recovered = false;
    if let Some(resugared) = resugar_symbol(&current) {
        current = Cow::Owned(resugared);
        recovered = true;
    }
    if let Some(resugared) = resugar_path(&current, origin == Origin::Untransformed) {
        current = Cow::Owned(resugared);
        recovered = true;
    }
    if let Some(resugared) = resugar_lists(&current) {
        current = Cow::Owned(resugared);
        recovered = true;
    }
    if let Some(rewritten) = rewrite_missing_fields(&current, has_field_impls) {
        current = Cow::Owned(rewritten);
        recovered = true;
    }

    if origin == Origin::Untransformed && !recovered {
        return None;
    }
    match current {
        Cow::Owned(text) => Some(text),
        Cow::Borrowed(_) => None,
    }
}

/// Post-process a message rustc built from several *styled fragments*, reading them as the one text
/// they render as. Returns the rewritten whole when reading them together achieves something
/// reading them singly cannot, and `None` to keep the fragments (and their styling) as they are.
///
/// Precondition: the caller has run [`postprocess_message`] over each fragment.
///
/// See `cgp-knowledge-base/cargo-cgp/implementation/error-processing.md`.
pub fn postprocess_fragments(
    fragments: &[&str],
    has_field_impls: bool,
    origin: Origin,
    ambiguous: &AmbiguousNames,
) -> Option<String> {
    if fragments.len() < 2 {
        return None;
    }
    let joined = fragments.concat();
    let rewritten = postprocess_message(&joined, has_field_impls, origin, ambiguous)?;
    (rewritten != joined).then_some(rewritten)
}
