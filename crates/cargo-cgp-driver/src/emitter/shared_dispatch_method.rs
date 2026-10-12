//! Pairing the two `E0428`s that one method two `#[cgp_auto_dispatch]` traits share produces.
//!
//! The macro names a method's helper function and its computer provider after the method alone, so
//! two dispatch traits in one module that declare `area` define `__compute_area__` and
//! `ComputeArea` twice, and name resolution reports an `E0428` for each, naming only the generated
//! items. The helper's caret carries the macro's expansion; the computer's, spanned on the method
//! name, does not. So neither alone identifies the mistake, but the pair does: a helper `E0428`
//! from a `#[cgp_auto_dispatch]` expansion and a computer `E0428` whose name the same method gives.
//! The macro defines the helper before the computer, so the helper's `E0428` is already buffered
//! when the computer's arrives. The pass needs no compiler query, since an `E0428` arrives during
//! name resolution.

use cargo_cgp_error_processing::{
    FIRST_DISPATCH_METHOD_LABEL, SECOND_DISPATCH_METHOD_LABEL, StructuralMistake,
    dispatch_computer_name, dispatch_helper_method, plan_structural_mistake,
    structural_mistake_help, structural_mistake_note,
};
use rustc_errors::codes::E0428;
use rustc_errors::{DiagArgValue, DiagInner, Sublevel};

use crate::emitter::buffer::BufEntry;
use crate::emitter::edit::{replace_header, subdiag, two_definitions_span};
use crate::resolve::is_from_auto_dispatch;

/// Pair `diag`, an incoming computer `E0428`, with the buffered helper `E0428` of the method two
/// `#[cgp_auto_dispatch]` traits share, and word the pair as one `[CGP-E061]` on `diag`, whose
/// carets are the two method names, dropping the helper's from `buffer`. Returns whether it paired.
pub(crate) fn pair_shared_dispatch_method(
    buffer: &mut Vec<BufEntry>,
    diag: &mut DiagInner,
) -> bool {
    let Some(name) = defined_twice(diag) else {
        return false;
    };
    let partner = buffer.iter().enumerate().find_map(|(index, entry)| {
        let BufEntry::Plain(buffered) = entry else {
            return None;
        };
        let helper = defined_twice(buffered)?;
        let method = helper_method(buffered, &helper)?;
        (dispatch_computer_name(&method) == name).then_some((index, helper, method))
    });
    let Some((index, helper, method)) = partner else {
        return false;
    };
    buffer.remove(index);
    word_shared_dispatch_method(diag, method, helper, name);
    true
}

/// The name an `E0428` ("the name `name` is defined multiple times") reports, read from its
/// argument, since the derived diagnostic carries no plain-string message.
fn defined_twice(diag: &DiagInner) -> Option<String> {
    if diag.code != Some(E0428) {
        return None;
    }
    match diag.args.get("name") {
        Some(DiagArgValue::Str(name)) => Some(name.to_string()),
        _ => None,
    }
}

/// The method `name` is the `#[cgp_auto_dispatch]` helper of, when it is one and `diag`'s caret
/// comes from that macro's expansion.
fn helper_method(diag: &DiagInner, name: &str) -> Option<String> {
    let method = dispatch_helper_method(name)?;
    let primary = diag.span.primary_span()?;
    is_from_auto_dispatch(primary).then(|| method.to_owned())
}

/// Reword `diag`, the computer's `E0428`, as the shared method, its two carets labelled as the two
/// method names.
fn word_shared_dispatch_method(
    diag: &mut DiagInner,
    method: String,
    helper: String,
    computer: String,
) {
    let mistake = StructuralMistake::SharedDispatchMethod {
        method,
        helper,
        computer,
    };
    replace_header(diag, plan_structural_mistake(&mistake));
    diag.span = two_definitions_span(
        &diag.span,
        FIRST_DISPATCH_METHOD_LABEL,
        SECOND_DISPATCH_METHOD_LABEL,
    );
    diag.children.clear();
    if let Some(note) = structural_mistake_note(&mistake) {
        diag.children.push(subdiag(Sublevel::Note, note));
    }
    diag.children
        .push(subdiag(Sublevel::Help, structural_mistake_help(&mistake)));
}
