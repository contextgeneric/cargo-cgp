//! The rustc-free model and wording for a `#[cgp_impl]` header that names the wrong trait.
//!
//! `#[cgp_impl(new RectangleArea)] impl AreaCalculator { … }` is the idiomatic way to write a
//! provider: the macro turns the header inside out into
//! `impl<__Context__> AreaCalculator<__Context__> for RectangleArea`, inserting the context as the
//! leading generic. Two mistakes put the wrong trait in that header, and they need different fixes:
//!
//! - **A consumer trait.** Naming the component's *consumer* trait `CanCalculateArea` where its
//!   *provider* trait `AreaCalculator` belongs. The macro then references a `CanCalculateAreaComponent`
//!   marker that does not exist and implements the wrong trait, so one mistake produces a burst of
//!   cryptic errors — E0425 (the missing marker), E0107 (the consumer trait takes one fewer generic
//!   than the inserted context supplies), E0186 (`&self` mismatch), E0207 (`__Context__`
//!   unconstrained) — plus a downstream check failure, none naming the real cause. The fix is to name
//!   the provider trait the component pairs the consumer with.
//! - **A non-CGP trait.** Applying `#[cgp_impl]` to a trait that is not a CGP component at all. There
//!   is no provider trait to point at; the fix is to make the trait a component with
//!   `#[cgp_component]`, or to drop `#[cgp_impl]` and write a plain `impl`.
//!
//! This model records which mistake it is, and [`plan_cgp_impl_misuse`] words it into the coded
//! header ([`CONSUMER_TRAIT_IN_PROVIDER_IMPL`] / [`NON_CGP_TRAIT_IN_CGP_IMPL`]) that replaces the raw
//! message, with [`cgp_impl_misuse_help`] carrying the fix. The driver's `resolve::cgp_impl_misuse`
//! module fills the model in from the live `TyCtxt` using the consumer- and provider-trait
//! fingerprints; keeping the wording here in owned `String` form makes it unit-testable without a
//! compiler.

use crate::code::{
    CONSUMER_TRAIT_IN_PROVIDER_BOUND, CONSUMER_TRAIT_IN_PROVIDER_IMPL, NON_CGP_TRAIT_IN_CGP_IMPL,
};

/// A trait named where a `#[cgp_impl]` provider trait belongs — in the impl header, or in a
/// higher-order provider's inner-provider bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CgpImplMisuse {
    /// The impl header names the component's *consumer* trait; `provider` is the provider trait to
    /// use instead (recovered from the consumer↔provider fingerprint).
    ConsumerTrait { consumer: String, provider: String },
    /// The impl header names a trait that is not a CGP component — there is no provider trait to
    /// suggest.
    NonCgpTrait { trait_name: String },
    /// An inner-provider bound (typically written through `#[use_provider]`) names the component's
    /// *consumer* trait where its *provider* trait belongs; `provider` is the provider trait to use.
    ConsumerProviderBound { consumer: String, provider: String },
}

/// Word a [`CgpImplMisuse`] into the coded header that replaces the raw `E0107`, naming the trait the
/// programmer wrote and — for a consumer trait — the provider trait to use instead of the generated
/// `__Context__` scaffolding the raw errors talk about.
pub fn plan_cgp_impl_misuse(misuse: &CgpImplMisuse) -> String {
    match misuse {
        CgpImplMisuse::ConsumerTrait { consumer, provider } => format!(
            "[{CONSUMER_TRAIT_IN_PROVIDER_IMPL}] `{consumer}` is a consumer trait, but a `#[cgp_impl]` \
             provider must implement its provider trait `{provider}`",
        ),
        CgpImplMisuse::NonCgpTrait { trait_name } => format!(
            "[{NON_CGP_TRAIT_IN_CGP_IMPL}] `#[cgp_impl]` can only implement a CGP component's provider \
             trait, but `{trait_name}` is not a CGP component",
        ),
        CgpImplMisuse::ConsumerProviderBound { consumer, provider } => format!(
            "[{CONSUMER_TRAIT_IN_PROVIDER_BOUND}] `{consumer}` is a consumer trait and cannot bound an \
             inner provider; a higher-order provider imports its provider trait `{provider}`",
        ),
    }
}

/// The `help` accompanying the header — the fix the raw errors never state.
pub fn cgp_impl_misuse_help(misuse: &CgpImplMisuse) -> String {
    match misuse {
        CgpImplMisuse::ConsumerTrait { consumer, provider } => format!(
            "change the impl header to target the provider trait: `impl {provider}` (not `impl {consumer}`)",
        ),
        CgpImplMisuse::NonCgpTrait { trait_name } => format!(
            "define `{trait_name}` as a component with `#[cgp_component]`, or drop `#[cgp_impl]` and \
             write a plain `impl` if it is an ordinary trait",
        ),
        CgpImplMisuse::ConsumerProviderBound { consumer, provider } => format!(
            "name the provider trait in the bound, idiomatically `#[use_provider(… : {provider})]` \
             (not the consumer trait `{consumer}`)",
        ),
    }
}

/// An inner provider a higher-order provider's body calls (`inner::method(self)`) without importing
/// it — the parameter carries no provider-trait bound. `inner` is the provider parameter's name and
/// `provider_trait` the trait it should be imported as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingUseProvider {
    pub inner: String,
    pub provider_trait: String,
}

/// Word a [`MissingUseProvider`] into the `[CGP-E016]` header that replaces the raw `E0599`, naming
/// the inner provider rather than the generated `__Context__` scaffolding rustc's suggestion leaks.
pub fn plan_missing_use_provider(missing: &MissingUseProvider) -> String {
    format!(
        "[{}] the inner provider `{}` is used but not imported",
        crate::code::INNER_PROVIDER_NOT_IMPORTED,
        missing.inner,
    )
}

/// The `help` accompanying the header — the fix the raw error never states: import the inner provider
/// with `#[use_provider]`, which supplies the leading context argument the provider trait needs.
pub fn missing_use_provider_help(missing: &MissingUseProvider) -> String {
    format!(
        "import it with `#[use_provider({}: {})]`",
        missing.inner, missing.provider_trait,
    )
}
