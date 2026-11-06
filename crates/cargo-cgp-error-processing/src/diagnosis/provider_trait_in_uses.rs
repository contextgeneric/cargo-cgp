//! The rustc-free model and wording for a provider trait named in `#[uses(…)]`.

use crate::syntax::with_arguments;

/// A provider trait named in `#[uses(…)]`, where a dependency on a component names its consumer
/// trait. `#[uses(AreaCalculator)]` emits `Self: AreaCalculator`, which lacks the context argument
/// the provider trait takes, so rustc reports missing generics and suggests the generated
/// `__Context__`; a generic component's provider trait written with its own arguments,
/// `#[uses(AreaCalculator<Rectangle>)]`, gets one argument too few, which rustc counts. `provider`
/// is the provider trait written and `consumer` the consumer trait it pairs with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderTraitInUses {
    /// The provider trait the attribute names.
    pub provider: String,
    /// The consumer trait the provider trait pairs with, which the dependency should name.
    pub consumer: String,
    /// The generic arguments the attribute gives the provider trait, as written, which the
    /// consumer trait takes in the same order; empty for a component without parameters.
    pub arguments: Vec<String>,
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

/// The `help` naming the consumer trait to declare instead, with the arguments the attribute
/// wrote, so the reader can type it as it stands.
pub fn provider_trait_in_uses_help(misuse: &ProviderTraitInUses) -> String {
    format!(
        "name the consumer trait instead: `#[uses({})]` (not the provider trait `{}`)",
        with_arguments(&misuse.consumer, &misuse.arguments),
        misuse.provider,
    )
}
