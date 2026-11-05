//! Two check tables in one module whose check traits share a name.

use cargo_cgp_error_processing::{CheckForm, CheckTraitName, StructuralMistake, invokes_macro};
use rustc_span::Span;
use rustc_span::hygiene::{ExpnKind, MacroKind};

use crate::config::{
    CHECK_COMPONENTS_MACRO, CHECK_COMPONENTS_TRAIT_PREFIX, DELEGATE_AND_CHECK_COMPONENTS_MACRO,
    DELEGATE_AND_CHECK_COMPONENTS_TRAIT_PREFIX,
};

/// The duplicate check trait an `E0428` ("the name `name` is defined multiple times") is about,
/// when both definitions, `first` (rustc's earlier definition) and `second` (the caret), come from
/// a `check_components!` or `delegate_and_check_components!` expansion, the only item those macros
/// define being the check trait. A definition the program wrote itself declines, so a table whose
/// check trait collides with the program's own trait keeps rustc's error. The name is read as the
/// one the macro derives only when both tables come from that macro and it carries the macro's
/// prefix. The test reads the spans' expansion data alone: an `E0428` comes from name resolution,
/// before any query that reads the HIR or the crate list is safe to run. `None` otherwise.
pub fn detect_duplicate_check_trait(
    name: &str,
    first: Span,
    second: Span,
) -> Option<StructuralMistake> {
    let first = check_macro_at(first)?;
    let second = check_macro_at(second)?;
    let prefix = match second {
        CheckForm::CheckComponents => CHECK_COMPONENTS_TRAIT_PREFIX,
        CheckForm::DelegateAndCheckComponents => DELEGATE_AND_CHECK_COMPONENTS_TRAIT_PREFIX,
    };
    let name = match name.strip_prefix(prefix) {
        Some(context) if !context.is_empty() && first == second => CheckTraitName::Derived {
            context: context.to_owned(),
        },
        _ => CheckTraitName::Given {
            check_trait: name.to_owned(),
        },
    };
    Some(StructuralMistake::DuplicateCheckTrait {
        name,
        first,
        second,
    })
}

/// The check macro whose expansion `span` comes from, or `None` for a span no check macro wrote.
/// `delegate_and_check_components!` is looked for first, in case its expansion passes through
/// `check_components!`.
fn check_macro_at(span: Span) -> Option<CheckForm> {
    let from = |macro_name: &str| {
        span.macro_backtrace().any(|expansion| {
            matches!(expansion.kind, ExpnKind::Macro(MacroKind::Bang, name)
                if invokes_macro(name.as_str(), macro_name))
        })
    };
    if from(DELEGATE_AND_CHECK_COMPONENTS_MACRO) {
        Some(CheckForm::DelegateAndCheckComponents)
    } else if from(CHECK_COMPONENTS_MACRO) {
        Some(CheckForm::CheckComponents)
    } else {
        None
    }
}
