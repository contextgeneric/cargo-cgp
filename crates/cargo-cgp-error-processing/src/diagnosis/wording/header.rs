//! The coded main-message headers of a resolved failure.

use crate::code::{
    ABSTRACT_TYPE_MISMATCH, CONSUMER_TRAIT_UNIMPLEMENTED, FIELD_TYPE_MISMATCH,
    WRAPPER_TRAIT_UNIMPLEMENTED,
};
use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::resolved::Resolved;
use crate::diagnosis::wording::lead::assoc_type_noun;

/// A `and`-joined, back-quoted list: `` `x` ``, `` `x` and `y` ``, or `` `x`, `y`, and `z` ``.
pub fn quoted_list(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| format!("`{item}`")).collect();
    match quoted.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The main message for a resolved failure: the trait(s) the context fails to implement, taken
/// from the typed resolution — which keys each component marker by its full path — so two same-named
/// components in different modules can never be confused. A CGP consumer trait reads as `the consumer
/// trait …` (`CGP-E001`); a hand-written wrapper trait (not a CGP consumer) reads as `the trait …`
/// (`CGP-E009`), chosen by [`Resolved::consumers_are_cgp`]. The subject is named `context \`X\`` when
/// it is the checked context, or plainly `` `X` `` when it is a foreign wrapper type that merely
/// holds the context (`Router<Arc<MockApp>>`), chosen by [`Resolved::subject_is_context`].
pub fn consumer_header(resolved: &Resolved) -> String {
    let (noun, verb) = if resolved.consumers.len() == 1 {
        ("trait", "is")
    } else {
        ("traits", "are")
    };
    let list = quoted_list(&resolved.consumers);
    // A foreign wrapper subject (`subject_is_context == false`) is named plainly, so it is not
    // mislabelled a context.
    let subject = if resolved.subject_is_context {
        format!("context `{}`", resolved.context)
    } else {
        format!("`{}`", resolved.context)
    };
    if resolved.consumers_are_cgp {
        format!(
            "[{CONSUMER_TRAIT_UNIMPLEMENTED}] the consumer {noun} {list} {verb} not implemented for {subject}"
        )
    } else {
        format!(
            "[{WRAPPER_TRAIT_UNIMPLEMENTED}] the {noun} {list} {verb} not implemented for {subject}"
        )
    }
}

/// The `[CGP-E003]` main message for a field-type mismatch: the field the wiring reads carries
/// the wrong type. The expected type comes from the failing projection and the actual from
/// querying the struct by `DefId`, so the two are read from the real type, never a same-named
/// one elsewhere.
pub fn field_mismatch_header(
    name: &str,
    owner: &str,
    expected: &str,
    expected_normalized: Option<&str>,
    actual: &str,
) -> String {
    let required = required_type(expected, expected_normalized);
    format!(
        "[{FIELD_TYPE_MISMATCH}] expected a `{name}` field of type {required} on `{owner}`, but found `{actual}`"
    )
}

/// Render a required type: the un-normalized form, followed by what it reduces to in parentheses
/// when that differs. Shared by the `[CGP-E003]` header and the `[CGP-E109]` leaf so the two cannot
/// state the same requirement two ways.
///
/// The un-normalized form leads because it names *where* the requirement comes from — a provider
/// reading `&Pool<Db>` under an imported abstract type requires
/// `Pool<<App as HasDbType>::Db>`, which points at the wiring — while the reduced form is what the
/// reader compares against the field, so neither alone is enough.
pub fn required_type(expected: &str, expected_normalized: Option<&str>) -> String {
    match expected_normalized {
        Some(normalized) => format!("`{expected}` (`{normalized}`)"),
        None => format!("`{expected}`"),
    }
}

/// The first field-type-mismatch cause of a resolution, if any — the leaf `[CGP-E003]` is
/// worded from.
pub fn mismatch_leaf(resolved: &Resolved) -> Option<&Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::FieldTypeMismatch { .. }))
}

/// The `[CGP-E017]` main message for an associated-type mismatch: the type the owner supplies is not
/// the one the wiring requires. Worded in the same shape as [`field_mismatch_header`] — required type
/// first, actual second — and reading `abstract type` for a CGP abstract-type component, where the
/// concrete type is a wiring choice, or `associated type` for any other trait.
pub fn assoc_mismatch_header(
    assoc: &str,
    trait_name: &str,
    owner: &str,
    expected: &str,
    expected_normalized: Option<&str>,
    actual: &str,
    component: Option<&str>,
) -> String {
    let noun = assoc_type_noun(component);
    let required = required_type(expected, expected_normalized);
    format!(
        "[{ABSTRACT_TYPE_MISMATCH}] expected the {noun} `{assoc}` of `{trait_name}` on `{owner}` to be {required}, but found `{actual}`"
    )
}

/// The first associated-type-mismatch cause of a resolution, if any — the leaf `[CGP-E017]` is
/// worded from.
pub fn assoc_mismatch_leaf(resolved: &Resolved) -> Option<&Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::AssocTypeMismatch { .. }))
}
