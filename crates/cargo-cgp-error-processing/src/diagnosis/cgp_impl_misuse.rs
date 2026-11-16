//! The rustc-free model and wording for a `#[cgp_impl]` header that names the wrong trait: a
//! consumer trait, or a trait that is no CGP component.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md and the `CGP-E032`
//! and `CGP-E033` entries of cgp-knowledge-base/cargo-cgp/error-code.md.

use crate::code::{
    CONSUMER_TRAIT_IN_PROVIDER_BOUND, CONSUMER_TRAIT_IN_PROVIDER_IMPL, NON_CGP_TRAIT_IN_CGP_IMPL,
};
use crate::syntax::with_arguments;

/// A trait named where a `#[cgp_impl]` provider trait belongs: in the impl header, or in a
/// higher-order provider's inner-provider bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CgpImplMisuse {
    /// The impl header names the component's *consumer* trait; `provider` is the provider trait to
    /// use instead (recovered from the consumer↔provider fingerprint).
    ConsumerTrait {
        /// The consumer trait the header names.
        consumer: String,
        /// The provider trait the header should name.
        provider: String,
        /// The generic arguments the header gives the consumer trait, as written, without the
        /// context the macro inserts; empty for a component without parameters.
        arguments: Vec<String>,
    },
    /// The impl header names a trait that is not a CGP component, so there is no provider trait to
    /// suggest.
    NonCgpTrait {
        /// The trait the header names.
        trait_name: String,
    },
    /// An inner-provider bound (typically written through `#[use_provider]`) names the component's
    /// *consumer* trait where its *provider* trait belongs; `provider` is the provider trait to
    /// use.
    ConsumerProviderBound {
        /// The consumer trait the bound names.
        consumer: String,
        /// The provider trait the bound should name.
        provider: String,
        /// The generic arguments the bound gives the consumer trait, as written, without the
        /// context the attribute inserts; empty for a component without parameters.
        arguments: Vec<String>,
    },
}

/// Word a [`CgpImplMisuse`] into the coded header that replaces the raw `E0107`, naming the trait
/// the programmer wrote and, for a consumer trait, the provider trait to use instead of the
/// generated `__Context__` scaffolding the raw errors talk about.
pub fn plan_cgp_impl_misuse(misuse: &CgpImplMisuse) -> String {
    match misuse {
        CgpImplMisuse::ConsumerTrait {
            consumer, provider, ..
        } => format!(
            "[{CONSUMER_TRAIT_IN_PROVIDER_IMPL}] `{consumer}` is a consumer trait, but a `#[cgp_impl]` \
             provider must implement its provider trait `{provider}`",
        ),
        CgpImplMisuse::NonCgpTrait { trait_name } => format!(
            "[{NON_CGP_TRAIT_IN_CGP_IMPL}] `#[cgp_impl]` can only implement a CGP component's provider \
             trait, but `{trait_name}` is not a CGP component",
        ),
        CgpImplMisuse::ConsumerProviderBound {
            consumer, provider, ..
        } => format!(
            "[{CONSUMER_TRAIT_IN_PROVIDER_BOUND}] `{consumer}` is a consumer trait and cannot bound an \
             inner provider; a higher-order provider imports its provider trait `{provider}`",
        ),
    }
}

/// The `help` accompanying the header: the fix the raw errors never state. A trait the fix names
/// carries the arguments the program wrote, so the reader can type it as it stands.
pub fn cgp_impl_misuse_help(misuse: &CgpImplMisuse) -> String {
    match misuse {
        CgpImplMisuse::ConsumerTrait {
            consumer,
            provider,
            arguments,
        } => format!(
            "change the impl header to target the provider trait: `impl {}` (not `impl {}`)",
            with_arguments(provider, arguments),
            with_arguments(consumer, arguments),
        ),
        CgpImplMisuse::NonCgpTrait { trait_name } => format!(
            "make `{trait_name}` a component, as in `#[cgp_component({trait_name}Provider)]`, and \
             implement its provider trait here (`impl {trait_name}Provider`), or drop \
             `#[cgp_impl]` and write a plain `impl` if it is an ordinary trait",
        ),
        CgpImplMisuse::ConsumerProviderBound {
            consumer,
            provider,
            arguments,
        } => format!(
            "name the provider trait in the bound, idiomatically `#[use_provider(… : {})]` (not the \
             consumer trait `{consumer}`)",
            with_arguments(provider, arguments),
        ),
    }
}
