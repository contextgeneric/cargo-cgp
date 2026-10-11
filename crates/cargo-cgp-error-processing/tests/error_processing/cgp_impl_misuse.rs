//! Tests for the `#[cgp_impl]` header-trait misuse wording.
//!
//! `plan_cgp_impl_misuse` and `cgp_impl_misuse_help` are pure functions over the rustc-free
//! [`CgpImplMisuse`] model, so they are driven directly over hand-built values — no compiler, no
//! diagnostic wrapper. The driver fills the same model in from the live `TyCtxt` using the consumer-
//! and provider-trait fingerprints. The two variants word to distinct codes and distinct fixes: a
//! consumer trait named where the provider trait belongs (`[CGP-E013]`, naming the provider to use),
//! and a trait that is not a CGP component at all (`[CGP-E014]`, with no provider to suggest).

use cargo_cgp_error_processing::{
    CgpImplMisuse, MissingUseProvider, cgp_impl_misuse_help, missing_use_provider_help,
    plan_cgp_impl_misuse, plan_missing_use_provider,
};

#[test]
fn consumer_trait_names_the_provider_to_use() {
    let misuse = CgpImplMisuse::ConsumerTrait {
        consumer: "CanCalculateArea".to_owned(),
        provider: "AreaCalculator".to_owned(),
    };
    assert_eq!(
        plan_cgp_impl_misuse(&misuse),
        "[CGP-E013] `CanCalculateArea` is a consumer trait, but a `#[cgp_impl]` provider must \
         implement its provider trait `AreaCalculator`",
    );
    assert_eq!(
        cgp_impl_misuse_help(&misuse),
        "change the impl header to target the provider trait: `impl AreaCalculator` (not `impl CanCalculateArea`)",
    );
}

#[test]
fn non_cgp_trait_has_no_provider_to_suggest() {
    let misuse = CgpImplMisuse::NonCgpTrait {
        trait_name: "Greet".to_owned(),
    };
    assert_eq!(
        plan_cgp_impl_misuse(&misuse),
        "[CGP-E014] `#[cgp_impl]` can only implement a CGP component's provider trait, but `Greet` \
         is not a CGP component",
    );
    assert_eq!(
        cgp_impl_misuse_help(&misuse),
        "define `Greet` as a component with `#[cgp_component]`, or drop `#[cgp_impl]` and write a \
         plain `impl` if it is an ordinary trait",
    );
}

#[test]
fn consumer_trait_in_a_provider_bound() {
    let misuse = CgpImplMisuse::ConsumerProviderBound {
        consumer: "CanCalculateArea".to_owned(),
        provider: "AreaCalculator".to_owned(),
    };
    assert_eq!(
        plan_cgp_impl_misuse(&misuse),
        "[CGP-E015] `CanCalculateArea` is a consumer trait and cannot bound an inner provider; a \
         higher-order provider imports its provider trait `AreaCalculator`",
    );
    assert_eq!(
        cgp_impl_misuse_help(&misuse),
        "name the provider trait in the bound, idiomatically `#[use_provider(… : AreaCalculator)]` \
         (not the consumer trait `CanCalculateArea`)",
    );
}

#[test]
fn inner_provider_not_imported() {
    let missing = MissingUseProvider {
        inner: "InnerCalculator".to_owned(),
        provider_trait: "AreaCalculator".to_owned(),
    };
    assert_eq!(
        plan_missing_use_provider(&missing),
        "[CGP-E016] the inner provider `InnerCalculator` is used but not imported",
    );
    assert_eq!(
        missing_use_provider_help(&missing),
        "import it with `#[use_provider(InnerCalculator: AreaCalculator)]`",
    );
}
