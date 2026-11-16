//! Pairing the two `E0428`s that one method two `#[cgp_auto_dispatch]` traits share produces.
//!
//! The macro names a method's helper function and its computer provider after the method alone, so
//! two dispatch traits in one module that declare `area` define `__compute_area__` and
//! `ComputeArea` twice, and name resolution reports an `E0428` for each, naming only the generated
//! items. The helper's caret carries the macro's expansion; the computer's, spanned on the method
//! name, does not. So neither alone identifies the mistake, but the pair does: a helper `E0428`
//! from a `#[cgp_auto_dispatch]` expansion and a computer `E0428` whose name the same method gives.
//! Whichever of the two arrives second finds the other in the buffer, and the pair is worded on the
//! computer's, whose carets are the method names. The macro defines the helper first, so the
//! computer's usually arrives second. The stage needs no compiler query, since an `E0428` arrives
//! during name resolution.

use cargo_cgp_error_processing::{
    ClassPlan, FIRST_DISPATCH_METHOD_LABEL, SECOND_DISPATCH_METHOD_LABEL, StructuralMistake,
    dispatch_computer_name, dispatch_helper_method,
};
use rustc_errors::codes::E0428;
use rustc_errors::{DiagArgValue, DiagInner};

use crate::detect::is_from_auto_dispatch;
use crate::emitter::buffer::BufEntry;
use crate::emitter::pipeline::{BufferEdit, Reshape, SpanPolicy, Verdict};

/// Pair `diag`, an incoming `E0428`, with the buffered `E0428` of the other half of the method two
/// `#[cgp_auto_dispatch]` traits share, and word the pair as one `[CGP-E061]` on the computer's
/// diagnostic, whose carets are the two method names. An incoming computer `E0428` pairs with a
/// buffered helper's, which is removed; an incoming helper's pairs with a buffered computer's,
/// which is worded in its place. Any other diagnostic is left to the next stage.
pub fn shared_dispatch_method(diag: &DiagInner, buffer: &[BufEntry]) -> Verdict {
    let Some(name) = defined_twice(diag) else {
        return Verdict::Continue;
    };
    let buffered_defined_twice = || {
        buffer
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                BufEntry::Plain(buffered) => Some((index, buffered, defined_twice(buffered)?)),
                _ => None,
            })
    };
    let helper_first = buffered_defined_twice().find_map(|(index, buffered, helper)| {
        let method = helper_method(buffered, &helper)?;
        (dispatch_computer_name(&method) == name)
            .then(|| (BufferEdit::Remove(index), method, helper, name.clone()))
    });
    let computer_first = || {
        let method = helper_method(diag, &name)?;
        let computer = dispatch_computer_name(&method);
        buffered_defined_twice().find_map(|(index, _, buffered)| {
            (buffered == computer).then(|| {
                (
                    BufferEdit::TakeAsSubject(index),
                    method.clone(),
                    name.clone(),
                    computer.clone(),
                )
            })
        })
    };
    let Some((buffer_edit, method, helper, computer)) = helper_first.or_else(computer_first) else {
        return Verdict::Continue;
    };
    let mistake = StructuralMistake::SharedDispatchMethod {
        method,
        helper,
        computer,
    };
    Verdict::Reshape(Reshape {
        records_spans: false,
        buffer_edit,
        ..Reshape::new(
            ClassPlan::structural_mistake(&mistake),
            SpanPolicy::TwoDefinitions {
                first: FIRST_DISPATCH_METHOD_LABEL,
                second: SECOND_DISPATCH_METHOD_LABEL,
            },
        )
    })
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
