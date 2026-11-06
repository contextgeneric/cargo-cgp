//! Tests for the `#[cgp_impl]` header-trait misuse wording.
//!
//! `plan_cgp_impl_misuse` and `cgp_impl_misuse_help` are pure functions over the rustc-free
//! [`CgpImplMisuse`] model, so they are driven directly over hand-built values, with no compiler
//! and no diagnostic wrapper. The driver fills the same model in from the live `TyCtxt` using the
//! consumer- and provider-trait fingerprints. The variants word to distinct codes and distinct
//! fixes: a consumer trait named where the provider trait belongs, in the header (`[CGP-E032]`) or
//! in an inner-provider bound (`[CGP-E034]`), each naming the provider trait to use, and a trait
//! that is not a CGP component at all (`[CGP-E033]`, with no provider trait to name).

use cargo_cgp_error_processing::{
    CgpImplMisuse, MissingUseProvider, ProviderTraitInUses, cgp_impl_misuse_help,
    missing_use_provider_help, plan_cgp_impl_misuse, plan_missing_use_provider,
    plan_provider_trait_in_uses, provider_trait_in_uses_help,
};

#[test]
fn consumer_trait_names_the_provider_to_use() {
    let misuse = CgpImplMisuse::ConsumerTrait {
        consumer: "CanCalculateArea".to_owned(),
        provider: "AreaCalculator".to_owned(),
        arguments: Vec::new(),
    };
    assert_eq!(
        plan_cgp_impl_misuse(&misuse),
        "[CGP-E032] `CanCalculateArea` is a consumer trait, but a `#[cgp_impl]` provider must \
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
        "[CGP-E033] `#[cgp_impl]` can only implement a CGP component's provider trait, but `Greet` \
         is not a CGP component",
    );
    assert_eq!(
        cgp_impl_misuse_help(&misuse),
        "make `Greet` a component, as in `#[cgp_component(GreetProvider)]`, and implement its \
         provider trait here (`impl GreetProvider`), or drop `#[cgp_impl]` and write a plain \
         `impl` if it is an ordinary trait",
    );
}

#[test]
fn consumer_trait_in_a_provider_bound() {
    let misuse = CgpImplMisuse::ConsumerProviderBound {
        consumer: "CanCalculateArea".to_owned(),
        provider: "AreaCalculator".to_owned(),
        arguments: Vec::new(),
    };
    assert_eq!(
        plan_cgp_impl_misuse(&misuse),
        "[CGP-E034] `CanCalculateArea` is a consumer trait and cannot bound an inner provider; a \
         higher-order provider imports its provider trait `AreaCalculator`",
    );
    assert_eq!(
        cgp_impl_misuse_help(&misuse),
        "name the provider trait in the bound, idiomatically `#[use_provider(… : AreaCalculator)]` \
         (not the consumer trait `CanCalculateArea`)",
    );
}

// Property: for all consumer-trait misuses with written arguments `args`, the help names the
// provider trait as `provider<args>`, in the order written, and a header help names the consumer
// trait it replaces as `consumer<args>`; with no arguments each name is bare, as the tests above
// check. A trait named without the arguments its component takes does not compile (`E0107`), so the
// help names what the reader can type.
#[test]
fn a_generic_component_is_named_with_the_arguments_written() {
    let arguments = vec!["'a".to_owned(), "Text".to_owned()];
    let header = CgpImplMisuse::ConsumerTrait {
        consumer: "CanRead".to_owned(),
        provider: "Reader".to_owned(),
        arguments: arguments.clone(),
    };
    assert_eq!(
        cgp_impl_misuse_help(&header),
        "change the impl header to target the provider trait: `impl Reader<'a, Text>` (not `impl \
         CanRead<'a, Text>`)",
    );
    let bound = CgpImplMisuse::ConsumerProviderBound {
        consumer: "CanRead".to_owned(),
        provider: "Reader".to_owned(),
        arguments,
    };
    assert_eq!(
        cgp_impl_misuse_help(&bound),
        "name the provider trait in the bound, idiomatically `#[use_provider(… : Reader<'a, Text>)]` \
         (not the consumer trait `CanRead`)",
    );
    assert_eq!(
        plan_cgp_impl_misuse(&bound),
        "[CGP-E034] `CanRead` is a consumer trait and cannot bound an inner provider; a \
         higher-order provider imports its provider trait `Reader`",
        "the header names the traits bare",
    );
}

#[test]
fn inner_provider_not_imported() {
    let missing = MissingUseProvider {
        inner: "InnerCalculator".to_owned(),
        provider_trait: Some("AreaCalculator".to_owned()),
        unresolved: Vec::new(),
    };
    assert_eq!(
        plan_missing_use_provider(&missing),
        "[CGP-E031] the inner provider `InnerCalculator` is used but not imported",
    );
    assert_eq!(
        missing_use_provider_help(&missing).as_deref(),
        Some("import it with `#[use_provider(InnerCalculator: AreaCalculator)]`"),
    );
}

#[test]
fn a_generic_component_is_imported_with_its_parameters() {
    let missing = MissingUseProvider {
        inner: "Inner".to_owned(),
        provider_trait: Some("AreaCalculator<Shape>".to_owned()),
        unresolved: Vec::new(),
    };
    assert_eq!(
        missing_use_provider_help(&missing).as_deref(),
        Some("import it with `#[use_provider(Inner: AreaCalculator<Shape>)]`"),
    );
    let unresolved = MissingUseProvider {
        inner: "Inner".to_owned(),
        provider_trait: Some("AreaCalculator<Shape>".to_owned()),
        unresolved: vec!["Shape".to_owned()],
    };
    assert_eq!(
        missing_use_provider_help(&unresolved).as_deref(),
        Some(
            "import it with `#[use_provider(Inner: AreaCalculator<Shape>)]`, with the type the \
             call uses in place of `Shape`"
        ),
    );
    let several = MissingUseProvider {
        inner: "Inner".to_owned(),
        provider_trait: Some("Combiner<A, B>".to_owned()),
        unresolved: vec!["A".to_owned(), "B".to_owned()],
    };
    assert_eq!(
        missing_use_provider_help(&several).as_deref(),
        Some(
            "import it with `#[use_provider(Inner: Combiner<A, B>)]`, with the types the call \
             uses in place of `A` and `B`"
        ),
    );
}

#[test]
fn an_inner_provider_whose_bound_cannot_be_written_gets_no_help() {
    let missing = MissingUseProvider {
        inner: "Inner".to_owned(),
        provider_trait: None,
        unresolved: Vec::new(),
    };
    assert_eq!(missing_use_provider_help(&missing), None);
}

#[test]
fn a_provider_trait_in_uses_names_the_consumer_trait_to_declare() {
    let misuse = ProviderTraitInUses {
        provider: "AreaCalculator".to_owned(),
        consumer: "CanCalculateArea".to_owned(),
    };
    assert_eq!(
        plan_provider_trait_in_uses(&misuse),
        "[CGP-E037] `AreaCalculator` is a provider trait and cannot be a `#[uses]` dependency; a \
         dependency names its consumer trait `CanCalculateArea`",
    );
    assert_eq!(
        provider_trait_in_uses_help(&misuse),
        "name the consumer trait instead: `#[uses(CanCalculateArea)]` (not the provider trait \
         `AreaCalculator`)",
    );
}
