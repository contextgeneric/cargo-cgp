//! Two `check_components!` tables in one module whose check traits share a name.

use cargo_cgp_error_processing::{CheckTraitName, StructuralMistake, invokes_macro};
use rustc_span::Span;
use rustc_span::hygiene::{ExpnKind, MacroKind};

use crate::config::{CHECK_COMPONENTS_MACRO, DELEGATE_AND_CHECK_COMPONENTS_MACRO};

/// The prefix `check_components!` puts before the context's last path segment to name the check
/// trait it derives.
const DERIVED_CHECK_TRAIT_PREFIX: &str = "__Check";

/// The duplicate check trait an `E0428` ("the name `name` is defined multiple times") at
/// `primary_span` is about, when the span comes from a `check_components!` (or
/// `delegate_and_check_components!`) expansion, the only item those macros define being the check
/// trait. The test reads the span's expansion data alone: an `E0428` comes from name resolution,
/// before any query that reads the HIR or the crate list is safe to run. `None` otherwise.
pub fn detect_duplicate_check_trait(name: &str, primary_span: Span) -> Option<StructuralMistake> {
    let from_check_macro = primary_span.macro_backtrace().any(|expansion| {
        matches!(expansion.kind, ExpnKind::Macro(MacroKind::Bang, macro_name)
            if invokes_macro(macro_name.as_str(), CHECK_COMPONENTS_MACRO)
                || invokes_macro(macro_name.as_str(), DELEGATE_AND_CHECK_COMPONENTS_MACRO))
    });
    if !from_check_macro {
        return None;
    }
    let name = match name.strip_prefix(DERIVED_CHECK_TRAIT_PREFIX) {
        Some(context) if !context.is_empty() => CheckTraitName::Derived {
            context: context.to_owned(),
        },
        _ => CheckTraitName::Given {
            check_trait: name.to_owned(),
        },
    };
    Some(StructuralMistake::DuplicateCheckTrait { name })
}
