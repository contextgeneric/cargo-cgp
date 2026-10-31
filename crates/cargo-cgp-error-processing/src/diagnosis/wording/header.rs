//! The class headline of a resolved failure.

use crate::code::{
    CONSUMER_TRAIT_UNIMPLEMENTED, DATA_OPERATION_UNIMPLEMENTED, PROVIDER_TRAIT_UNIMPLEMENTED,
    WRAPPER_TRAIT_UNIMPLEMENTED,
};
use crate::diagnosis::resolved::{Resolved, ResolvedClass};
use crate::diagnosis::wording::quoted_list;

/// The class sentence of a resolved failure: the trait(s) the context fails to implement, taken
/// from the typed resolution, which keys each component marker by its full path, so two same-named
/// components in different modules can never be confused. A CGP consumer trait reads as
/// `the consumer trait …`; a trait that is not a CGP consumer (a hand-written wrapper, or a
/// `#[cgp_fn]` blanket trait) reads as `the trait …`, chosen by [`Resolved::class`].
/// The subject is named `` context `X` `` when it is the checked context, or plainly `` `X` `` when
/// it is a foreign type that merely holds the context (`Router<Arc<MockApp>>`), chosen by
/// [`Resolved::subject_is_context`]. A `#[check_providers]` entry, or a provider's own method called
/// directly, reads as the provider trait the provider fails to implement for the context.
pub fn class_sentence(resolved: &Resolved) -> String {
    if let ResolvedClass::Provider(provider) = &resolved.class {
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
    let kind = if resolved.class == ResolvedClass::Consumer {
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
    match resolved.class {
        ResolvedClass::Provider(_) => PROVIDER_TRAIT_UNIMPLEMENTED,
        ResolvedClass::DataOperation => DATA_OPERATION_UNIMPLEMENTED,
        ResolvedClass::Consumer => CONSUMER_TRAIT_UNIMPLEMENTED,
        ResolvedClass::Wrapper => WRAPPER_TRAIT_UNIMPLEMENTED,
    }
}

/// The class headline of a resolved failure: its [`class_code`] and [`class_sentence`]. It is the
/// internal headline that matches a re-report the resolver declined (whose text rewrite produces
/// the same sentence), and the displayed one only when the causes are too mixed to state at once
/// (see [`cause_headline`](super::cause_headline)).
pub fn consumer_header(resolved: &Resolved) -> String {
    format!("[{}] {}", class_code(resolved), class_sentence(resolved))
}
