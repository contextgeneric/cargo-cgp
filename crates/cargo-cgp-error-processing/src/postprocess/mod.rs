//! Post-processing text transforms for CGP diagnostics.
//!
//! These are the compiler-free string transforms the *driver* applies to a diagnostic it
//! did not rewrite, so raw CGP constructs (`cgp::` path prefixes, `Symbol!` spines, unmet
//! `HasField` bounds) do not look confusing in an error the tool leaves otherwise
//! untouched. Each transform is `&str -> Option<String>` — `Some` when it changed the text,
//! `None` when it left it alone — so a caller can tell whether anything matched. They live
//! here, apart from the driver, so they build and are unit-tested on any toolchain without
//! the driver's `rustc_private` linkage; the driver applies [`postprocess_message`] to each
//! message string of a `DiagInner` on the fallback path.

mod missing_field;
mod resugar_symbol;
mod strip_prefixes;

use std::borrow::Cow;

pub use missing_field::*;
pub use resugar_symbol::*;
pub use strip_prefixes::*;

/// Run the post-processing chain over one message string: strip CGP path prefixes, resugar
/// `Symbol!`, then rewrite an unmet `HasField` bound. The order matters — prefix stripping
/// first so the later stages match the bare CGP names, `Symbol!` resugaring before the
/// field rewrite (which matches the resugared `HasField<Symbol!("…")>` form). Returns the
/// rewritten text when any stage changed it, `None` otherwise.
///
/// `has_field_impls` is the whole-diagnostic fact the field rewrite needs (whether the
/// context implements `HasField` for any field), which the caller computes once with
/// [`context_has_hasfield_impls`] across every message before rewriting each.
pub fn postprocess_message(text: &str, has_field_impls: bool) -> Option<String> {
    let mut current: Cow<str> = Cow::Borrowed(text);
    if let Some(stripped) = strip_cgp_prefixes(&current) {
        current = Cow::Owned(stripped);
    }
    if let Some(resugared) = resugar_symbol(&current) {
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
