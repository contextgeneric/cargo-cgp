//! The post-processing chain the driver applies to a diagnostic's messages.

use std::borrow::Cow;

use crate::postprocess::missing_field::rewrite_missing_fields;
use crate::postprocess::resugar_path::resugar_path;
use crate::postprocess::resugar_symbol::resugar_symbol;
use crate::postprocess::strip_prefixes::strip_cgp_prefixes;

/// Run the post-processing chain over one message string: strip CGP path prefixes, resugar
/// `Symbol!`, resugar `Path!`, then rewrite an unmet `HasField` bound. The order matters —
/// prefix stripping first so the later stages match the bare CGP names, `Symbol!` resugaring
/// before `Path!` resugaring (which reads the already-resugared `Symbol!("…")` segments) and
/// before the field rewrite (which matches the resugared `HasField<Symbol!("…")>` form).
/// Returns the rewritten text when any stage changed it, `None` otherwise.
///
/// `has_field_impls` is the whole-diagnostic fact the field rewrite needs (whether the
/// context implements `HasField` for any field), which the caller computes once with
/// [`context_has_hasfield_impls`](super::context_has_hasfield_impls) across every message
/// before rewriting each.
pub fn postprocess_message(text: &str, has_field_impls: bool) -> Option<String> {
    let mut current: Cow<str> = Cow::Borrowed(text);
    if let Some(stripped) = strip_cgp_prefixes(&current) {
        current = Cow::Owned(stripped);
    }
    if let Some(resugared) = resugar_symbol(&current) {
        current = Cow::Owned(resugared);
    }
    if let Some(resugared) = resugar_path(&current) {
        current = Cow::Owned(resugared);
    }
    if let Some(rewritten) = rewrite_missing_fields(&current, has_field_impls) {
        current = Cow::Owned(rewritten);
    }
    match current {
        Cow::Owned(text) => Some(text),
        Cow::Borrowed(_) => None,
    }
}
