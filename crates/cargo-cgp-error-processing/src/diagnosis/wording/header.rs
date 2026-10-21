//! The class headline of a resolved failure, and the helpers the cause sentences share.

use crate::code::{
    CONSUMER_TRAIT_UNIMPLEMENTED, DATA_OPERATION_UNIMPLEMENTED, PROVIDER_TRAIT_UNIMPLEMENTED,
    WRAPPER_TRAIT_UNIMPLEMENTED,
};
use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::resolved::Resolved;

/// An `and`-joined, back-quoted list: `` `x` ``, `` `x` and `y` ``, or `` `x`, `y`, and `z` ``.
pub fn quoted_list(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| format!("`{item}`")).collect();
    match quoted.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The class sentence of a resolved failure: the trait(s) the context fails to implement, taken
/// from the typed resolution, which keys each component marker by its full path, so two same-named
/// components in different modules can never be confused. A CGP consumer trait reads as
/// `the consumer trait …`; a trait that is not a CGP consumer (a hand-written wrapper, or a
/// `#[cgp_fn]` blanket trait) reads as `the trait …`, chosen by [`Resolved::consumers_are_cgp`].
/// The subject is named `context \`X\`` when it is the checked context, or plainly `` `X` `` when
/// it is a foreign type that merely holds the context (`Router<Arc<MockApp>>`), chosen by
/// [`Resolved::subject_is_context`]. A `#[check_providers]` entry, or a provider's own method called
/// directly, reads as the provider trait the provider fails to implement for the context.
pub fn class_sentence(resolved: &Resolved) -> String {
    if let Some(provider) = &resolved.provider {
        return format!(
            "the provider trait {} with context `{}` is not implemented for provider `{provider}`",
            quoted_list(&resolved.consumers),
            resolved.context,
        );
    }
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
    let kind = if resolved.consumers_are_cgp {
        "consumer "
    } else {
        ""
    };
    format!("the {kind}{noun} {list} {verb} not implemented for {subject}")
}

/// The class code of a resolved failure, chosen by the anchor that matched the caret: `CGP-E002`
/// for a `#[check_providers]` entry or a provider's own method called directly, `CGP-E050` for a
/// call to one of CGP's extensible-data traits,
/// `CGP-E001` for a CGP consumer trait, and `CGP-E003` for any other trait.
pub fn class_code(resolved: &Resolved) -> &'static str {
    if resolved.provider.is_some() {
        PROVIDER_TRAIT_UNIMPLEMENTED
    } else if resolved.data_operation {
        DATA_OPERATION_UNIMPLEMENTED
    } else if resolved.consumers_are_cgp {
        CONSUMER_TRAIT_UNIMPLEMENTED
    } else {
        WRAPPER_TRAIT_UNIMPLEMENTED
    }
}

/// The class headline of a resolved failure: its [`class_code`] and [`class_sentence`]. It is the
/// internal headline that matches a re-report the resolver declined (whose text rewrite produces
/// the same sentence), and the displayed one only when the causes are too mixed to state at once
/// (see [`cause_headline`](super::cause_headline)).
pub fn consumer_header(resolved: &Resolved) -> String {
    format!("[{}] {}", class_code(resolved), class_sentence(resolved))
}

/// Render a required type: the un-normalized form, followed by what it reduces to in parentheses
/// when that differs. Both mismatch cause sentences render their required type through it.
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

/// The first field-type-mismatch cause of a resolution, if any — the leaf `[CGP-E004]` is
/// worded from.
pub fn mismatch_leaf(resolved: &Resolved) -> Option<&Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::FieldTypeMismatch { .. }))
}

/// The first associated-type-mismatch cause of a resolution, if any — the leaf `[CGP-E005]` is
/// worded from.
pub fn assoc_mismatch_leaf(resolved: &Resolved) -> Option<&Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::AssocTypeMismatch { .. }))
}

/// The first variant-payload-mismatch cause of a resolution, if any: the extensible-data
/// counterpart of [`mismatch_leaf`], which an `E0271` block states as its headline.
pub fn variant_mismatch_leaf(resolved: &Resolved) -> Option<&Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::VariantTypeMismatch { .. }))
}
