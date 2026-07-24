//! The post-processing chain the driver applies to a diagnostic's messages.

use std::borrow::Cow;

use crate::postprocess::missing_field::rewrite_missing_fields;
use crate::postprocess::resugar_list::resugar_lists;
use crate::postprocess::resugar_path::resugar_path;
use crate::postprocess::resugar_symbol::resugar_symbol;
use crate::postprocess::strip_modules::strip_module_paths;
use crate::postprocess::strip_prefixes::strip_cgp_prefixes;

/// Run the post-processing chain over one message string: strip CGP path prefixes, resugar
/// `Symbol!`, resugar `Path!`, resugar `Product!`/`Sum!` (and their `Struct!`/`Enum!` forms), then
/// rewrite an unmet `HasField` bound. The order matters — prefix stripping first so the later stages
/// match the bare CGP names, `Symbol!` resugaring before `Path!` resugaring (which reads the
/// already-resugared `Symbol!("…")` segments) and before the list resugaring (which reads a `Field`'s
/// `Symbol!("…")` tag when naming a struct field or enum variant), and before the field rewrite
/// (which matches the resugared `HasField<Symbol!("…")>` form). Returns the rewritten text when any
/// stage changed it, `None` otherwise.
///
/// `has_field_impls` is the whole-diagnostic fact the field rewrite needs (whether the
/// context implements `HasField` for any field), which the caller computes once with
/// [`context_has_hasfield_impls`](super::context_has_hasfield_impls) across every message
/// before rewriting each.
///
/// `bare_paths` chooses the `Path!` form: a rewritten diagnostic (the tool constructed the message)
/// shows a bare `@…` path, while an un-rewritten resugaring fallback shows the `Path!(@…)` macro
/// form — so the caller passes `true` on the fallback and `false` on a rewrite.
pub fn postprocess_message(text: &str, has_field_impls: bool, bare_paths: bool) -> Option<String> {
    let mut current: Cow<str> = Cow::Borrowed(text);
    // Strip module qualifiers first (subsuming the CGP-prefix strip), so the resugaring stages
    // below match the bare `Symbol`/`Chars`/`PathCons` names and the output carries no module noise.
    if let Some(stripped) = strip_module_paths(&current) {
        current = Cow::Owned(stripped);
    }
    if let Some(stripped) = strip_cgp_prefixes(&current) {
        current = Cow::Owned(stripped);
    }
    if let Some(resugared) = resugar_symbol(&current) {
        current = Cow::Owned(resugared);
    }
    if let Some(resugared) = resugar_path(&current, !bare_paths) {
        current = Cow::Owned(resugared);
    }
    if let Some(resugared) = resugar_lists(&current) {
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
    bare_paths: bool,
) -> Option<String> {
    if fragments.len() < 2 {
        return None;
    }
    let joined = fragments.concat();
    let rewritten = postprocess_message(&joined, has_field_impls, bare_paths)?;
    (rewritten != joined).then_some(rewritten)
}
