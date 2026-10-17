//! Tests for the wording of a provider-trait impl written without `#[cgp_provider]` (`[CGP-E039]`).

use cargo_cgp_error_processing::{
    ProviderImplWithoutMacro, plan_provider_impl_without_macro, provider_impl_without_macro_help,
};

#[test]
fn a_provider_impl_without_the_macro_names_the_provider_and_the_fix() {
    let mistake = ProviderImplWithoutMacro {
        provider: "GreetHello".to_owned(),
        provider_trait: "Greeter".to_owned(),
    };
    assert_eq!(
        plan_provider_impl_without_macro(&mistake),
        "[CGP-E039] the impl of `Greeter` for `GreetHello` is written without `#[cgp_provider]`",
    );
    assert_eq!(
        provider_impl_without_macro_help(&mistake),
        "add `#[cgp_provider]` to the impl of `Greeter`, or write the provider with `#[cgp_impl]`",
    );
}
