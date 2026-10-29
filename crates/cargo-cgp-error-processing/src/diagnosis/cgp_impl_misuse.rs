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
//!   cryptic errors (E0425 for the missing marker, E0107 because the consumer trait takes one fewer
//!   generic than the inserted context supplies, E0186 for the `&self` mismatch, E0207 for the
//!   unconstrained `__Context__`) plus a downstream check failure, none naming the real cause. The
//!   fix is to name the provider trait the component pairs the consumer with.
//! - **A non-CGP trait.** Applying `#[cgp_impl]` to a trait that is not a CGP component at all. There
//!   is no provider trait to point at. The fix is to make the trait a component and name the
//!   provider trait it generates in the header (making it a component alone leaves a consumer trait
//!   in the header), or to drop `#[cgp_impl]` and write a plain `impl`.
//!
//! This model records which mistake it is, and [`plan_cgp_impl_misuse`] words it into the coded
//! header ([`CONSUMER_TRAIT_IN_PROVIDER_IMPL`] / [`NON_CGP_TRAIT_IN_CGP_IMPL`]) that replaces the raw
//! message, with [`cgp_impl_misuse_help`] carrying the fix. The driver's `detect::cgp_impl_misuse`
//! module fills the model in from the live `TyCtxt` using the consumer- and provider-trait
//! fingerprints; keeping the wording here in owned `String` form makes it unit-testable without a
//! compiler.

use crate::code::{
    CONSUMER_TRAIT_IN_PROVIDER_BOUND, CONSUMER_TRAIT_IN_PROVIDER_IMPL, NON_CGP_TRAIT_IN_CGP_IMPL,
};
use crate::diagnosis::wording::quoted_list;

/// A trait named where a `#[cgp_impl]` provider trait belongs: in the impl header, or in a
/// higher-order provider's inner-provider bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CgpImplMisuse {
    /// The impl header names the component's *consumer* trait; `provider` is the provider trait to
    /// use instead (recovered from the consumer↔provider fingerprint).
    ConsumerTrait { consumer: String, provider: String },
    /// The impl header names a trait that is not a CGP component, so there is no provider trait to
    /// suggest.
    NonCgpTrait { trait_name: String },
    /// An inner-provider bound (typically written through `#[use_provider]`) names the component's
    /// *consumer* trait where its *provider* trait belongs; `provider` is the provider trait to use.
    ConsumerProviderBound { consumer: String, provider: String },
}

/// Word a [`CgpImplMisuse`] into the coded header that replaces the raw `E0107`, naming the trait the
/// programmer wrote and, for a consumer trait, the provider trait to use instead of the generated
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

/// The `help` accompanying the header: the fix the raw errors never state.
pub fn cgp_impl_misuse_help(misuse: &CgpImplMisuse) -> String {
    match misuse {
        CgpImplMisuse::ConsumerTrait { consumer, provider } => format!(
            "change the impl header to target the provider trait: `impl {provider}` (not `impl {consumer}`)",
        ),
        CgpImplMisuse::NonCgpTrait { trait_name } => format!(
            "make `{trait_name}` a component, as in `#[cgp_component({trait_name}Provider)]`, and \
             implement its provider trait here (`impl {trait_name}Provider`), or drop \
             `#[cgp_impl]` and write a plain `impl` if it is an ordinary trait",
        ),
        CgpImplMisuse::ConsumerProviderBound { consumer, provider } => format!(
            "name the provider trait in the bound, idiomatically `#[use_provider(… : {provider})]` \
             (not the consumer trait `{consumer}`)",
        ),
    }
}

/// An inner provider a higher-order provider's body calls (`inner::method(self)`) without importing
/// it, so the parameter carries no provider-trait bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingUseProvider {
    /// The inner provider parameter's name.
    pub inner: String,
    /// The provider trait to import it as, as `#[use_provider]` writes it: with the component's
    /// parameters after the context, which the attribute supplies (`AreaCalculator<Shape>`), each
    /// one the call does not give away written as the trait's own parameter name. `None` when the
    /// tool cannot write the bound, as for a trait with a lifetime or const parameter.
    pub provider_trait: Option<String>,
    /// The provider trait's parameters the call does not give away, named in `provider_trait` for
    /// the reader to replace.
    pub unresolved: Vec<String>,
}

/// Word a [`MissingUseProvider`] into the `[CGP-E031]` header that replaces the raw `E0599`, naming
/// the inner provider rather than the generated `__Context__` scaffolding rustc's suggestion leaks.
pub fn plan_missing_use_provider(missing: &MissingUseProvider) -> String {
    format!(
        "[{}] the inner provider `{}` is used but not imported",
        crate::code::INNER_PROVIDER_NOT_IMPORTED,
        missing.inner,
    )
}

/// The `help` accompanying the header, the fix the raw error never states: import the inner
/// provider with `#[use_provider]`, which supplies the leading context argument the provider trait
/// needs. `None` when the bound cannot be written.
pub fn missing_use_provider_help(missing: &MissingUseProvider) -> Option<String> {
    let provider_trait = missing.provider_trait.as_ref()?;
    let import = format!(
        "import it with `#[use_provider({}: {provider_trait})]`",
        missing.inner,
    );
    Some(match missing.unresolved.len() {
        0 => import,
        1 => format!(
            "{import}, with the type the call uses in place of {}",
            quoted_list(&missing.unresolved),
        ),
        _ => format!(
            "{import}, with the types the call uses in place of {}",
            quoted_list(&missing.unresolved),
        ),
    })
}

/// A provider trait named in `#[uses(…)]`, where a dependency on a component names its consumer
/// trait. `#[uses(AreaCalculator)]` emits `Self: AreaCalculator`, which lacks the context argument
/// the provider trait takes, so rustc reports missing generics and suggests the generated
/// `__Context__`. `provider` is the provider trait written and `consumer` the consumer trait it pairs
/// with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderTraitInUses {
    pub provider: String,
    pub consumer: String,
}

/// Word a [`ProviderTraitInUses`] into the `[CGP-E037]` header that replaces the raw `E0107`.
pub fn plan_provider_trait_in_uses(misuse: &ProviderTraitInUses) -> String {
    format!(
        "[{}] `{}` is a provider trait and cannot be a `#[uses]` dependency; a dependency names its \
         consumer trait `{}`",
        crate::code::PROVIDER_TRAIT_IN_USES,
        misuse.provider,
        misuse.consumer,
    )
}

/// The `help` naming the consumer trait to declare instead.
pub fn provider_trait_in_uses_help(misuse: &ProviderTraitInUses) -> String {
    format!(
        "name the consumer trait instead: `#[uses({})]` (not the provider trait `{}`)",
        misuse.consumer, misuse.provider,
    )
}
