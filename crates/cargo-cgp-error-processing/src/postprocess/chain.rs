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
    /// `HasField` bound), so a plain Rust message stays exactly as rustc printed it; a path shows in
    /// its `Path!(@…)` form, since the reader meets it as a type.
    Untransformed,
}

/// Run the post-processing chain over one message string: strip module qualifiers except on a name
/// in `ambiguous`, resugar `Symbol!`, resugar `Path!`, resugar `Product!`/`Sum!` (and their
/// `Struct!`/`Enum!` forms), then rewrite an unmet `HasField` bound. The order matters — the strips
/// first so the later stages match the bare CGP names, `Symbol!` resugaring before `Path!`
/// resugaring (which reads the already-resugared `Symbol!("…")` segments) and before the list
/// resugaring (which reads a `Field`'s `Symbol!("…")` tag when naming a struct field or enum
/// variant), and before the field rewrite (which matches the resugared `HasField<Symbol!("…")>`
/// form). Returns the rewritten text when the chain changed it, `None` otherwise.
///
/// For an [`Origin::Untransformed`] message the strips alone never count as a change: the result
/// is kept only when a later stage recovered a CGP encoding, so a glob-imported CGP trait keeps
/// the path rustc printed.
///
/// `ambiguous` holds the type names that name more than one item in the build; a path ending in one
/// keeps its qualifier everywhere, so `std::io::Error` never reads as another `Error`.
///
/// `has_field_impls` is the whole-diagnostic fact the field rewrite needs (whether the
/// context implements `HasField` for any field), which the caller computes once with
/// [`context_has_hasfield_impls`](super::context_has_hasfield_impls) across every message
/// before rewriting each.
pub fn postprocess_message(
    text: &str,
    has_field_impls: bool,
    origin: Origin,
    ambiguous: &AmbiguousNames,
) -> Option<String> {
    let mut current: Cow<str> = Cow::Borrowed(text);
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

/// Post-process a message rustc built from several *styled fragments*, reading them as the one
/// text they render as. Returns the rewritten whole when reading them together achieves something
/// reading them singly cannot, and `None` to keep the fragments — and their styling — as they are.
///
/// rustc splits a message into fragments to highlight parts of it, and when it highlights the
/// *difference* between two types it splits at every difference. Its "similar impl" hint does
/// exactly that, so a CGP type in one is shredded — `Symbol<3, Chars<'B', …>>` becomes a fragment
/// per character — and no fragment holds a whole construct for [`postprocess_message`] to match.
/// The rendered line is the concatenation, so matching on the concatenation is matching on what the
/// reader actually sees.
///
/// The caller runs [`postprocess_message`] over each fragment first and passes the results here, so
/// this fires only on a construct that genuinely spans a boundary. That is what makes flattening
/// safe to pay for: the caller replaces the fragments with this one string, losing rustc's
/// highlighting, and it only does so when there was a CGP construct to recover — never merely
/// because a fragment was tidied on its own.
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
