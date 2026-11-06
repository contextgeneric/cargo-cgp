//! Two `#[cgp_auto_dispatch]` traits in one module that declare a method of one name.

use cargo_cgp_error_processing::invokes_macro;
use rustc_span::Span;
use rustc_span::hygiene::{ExpnKind, MacroKind};

use crate::config::CGP_AUTO_DISPATCH_MACRO;

/// Whether `span`, the caret of an `E0428`, comes from a `#[cgp_auto_dispatch]` expansion, read
/// from the span's expansion data alone: an `E0428` comes from name resolution, before any query
/// that reads the HIR is safe to run. The helper function the macro generates for a method carries
/// the expansion; its computer provider is spanned on the method name the programmer wrote, which
/// carries none.
pub fn is_from_auto_dispatch(span: Span) -> bool {
    span.macro_backtrace().any(|expansion| {
        matches!(expansion.kind, ExpnKind::Macro(MacroKind::Attr, macro_name)
            if invokes_macro(macro_name.as_str(), CGP_AUTO_DISPATCH_MACRO))
    })
}
