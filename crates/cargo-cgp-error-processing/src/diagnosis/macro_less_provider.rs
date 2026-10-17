//! The rustc-free model and wording for a provider-trait impl written without its macro.
//!
//! `#[cgp_provider]` (and `#[cgp_impl]`, which lowers to it) writes the provider's impl of the
//! provider trait together with the impl of `IsProviderFor` the provider trait requires as a
//! supertrait. An impl written by hand, without either macro, has no such marker impl, so the
//! compiler rejects it on its own supertrait bound, with a list of every unrelated implementor of
//! `IsProviderFor` beside it. The driver recognizes the shape off the compiler, and
//! [`plan_provider_impl_without_macro`] words the coded header with
//! [`provider_impl_without_macro_help`] carrying the fix.

use crate::code::PROVIDER_IMPL_WITHOUT_MACRO;

/// A provider-trait impl on a provider struct that has no `IsProviderFor` impl for the trait's
/// component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderImplWithoutMacro {
    /// The provider struct the impl is written for, e.g. `GreetHello`.
    pub provider: String,
    /// The provider trait the impl implements, e.g. `Greeter`.
    pub provider_trait: String,
}

/// The coded header that replaces the compiler's message about the missing `IsProviderFor`.
pub fn plan_provider_impl_without_macro(mistake: &ProviderImplWithoutMacro) -> String {
    format!(
        "[{PROVIDER_IMPL_WITHOUT_MACRO}] the impl of `{}` for `{}` is written without `#[cgp_provider]`",
        mistake.provider_trait, mistake.provider,
    )
}

/// The `help` naming the fix: the macro that writes the marker impl with the provider impl.
pub fn provider_impl_without_macro_help(mistake: &ProviderImplWithoutMacro) -> String {
    format!(
        "add `#[cgp_provider]` to the impl of `{}`, or write the provider with `#[cgp_impl]`",
        mistake.provider_trait,
    )
}
