//! Dropping a suggestion that splices code right after an attribute macro.
//!
//! When an attribute macro's expansion fails to type-check, rustc may suggest a fix at a span the
//! expansion carries, and an attribute macro's generated code carries the attribute's own span.
//! A suggested insertion at the end of that span lands between the attribute's `]` and the item
//! (`#[cgp_impl(new SquareArea)]<Shape>`, `#[cgp_auto_getter].as_deref()`), where no edit of
//! the user's source can repair what the macro generated. Such a suggestion is wrong whatever the
//! error, so the drop leaves the diagnostic's other suggestions in place.

use rustc_errors::{CodeSuggestion, DiagInner};
use rustc_span::MacroKind;
use rustc_span::hygiene::ExpnKind;

use crate::emitter::edit::retain_suggestions;

/// Drop each structured suggestion of `diag` with an insertion at the end of an attribute macro's
/// call site, the point right after the attribute. Call it only for a crate that uses CGP, so a
/// plain Rust error keeps every suggestion.
pub(crate) fn drop_attribute_splices(diag: &mut DiagInner) {
    retain_suggestions(diag, |suggestion| !splices_after_attribute(suggestion));
}

/// Whether a substitution of `suggestion` inserts text, at a span an attribute macro's expansion
/// carries, exactly where that macro's call site ends.
fn splices_after_attribute(suggestion: &CodeSuggestion) -> bool {
    suggestion
        .substitutions
        .iter()
        .flat_map(|substitution| &substitution.parts)
        .any(|part| {
            if !part.span.is_empty() || !part.span.from_expansion() {
                return false;
            }
            let expansion = part.span.ctxt().outer_expn_data();
            matches!(expansion.kind, ExpnKind::Macro(MacroKind::Attr, _))
                && expansion.call_site.hi() == part.span.lo()
        })
}
