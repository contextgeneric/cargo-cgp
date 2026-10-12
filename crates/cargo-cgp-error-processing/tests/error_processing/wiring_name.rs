//! Tests for the wording of a wiring key or value that does not resolve (`[CGP-E038]`,
//! `[CGP-E040]` to `[CGP-E043]`).
//!
//! `plan_unresolved_wiring_name` and `unresolved_wiring_name_help` are pure functions over the
//! rustc-free [`UnresolvedWiringName`] model, driven here over hand-built values.

use cargo_cgp_error_processing::{
    UnresolvedWiringName, plan_unresolved_wiring_name, unresolved_wiring_name_help,
};

#[test]
fn a_consumer_trait_as_a_key_names_its_component() {
    let name = UnresolvedWiringName::ConsumerTraitAsKey {
        consumer: "CanGreet".to_owned(),
        component: "GreeterComponent".to_owned(),
    };
    assert_eq!(
        plan_unresolved_wiring_name(&name),
        "[CGP-E040] the wiring key `CanGreet` is a consumer trait, but a key names a component",
    );
    assert_eq!(
        unresolved_wiring_name_help(&name).as_deref(),
        Some("name its component instead: `GreeterComponent`"),
    );
}

#[test]
fn a_component_not_in_scope_names_its_import() {
    let name = UnresolvedWiringName::ComponentNotImported {
        component: "ErrorTypeProviderComponent".to_owned(),
        path: "cgp::core::error::ErrorTypeProviderComponent".to_owned(),
    };
    assert_eq!(
        plan_unresolved_wiring_name(&name),
        "[CGP-E041] the component `ErrorTypeProviderComponent` is not in scope",
    );
    assert_eq!(
        unresolved_wiring_name_help(&name).as_deref(),
        Some("import it with `use cgp::core::error::ErrorTypeProviderComponent;`"),
    );
}

#[test]
fn a_key_guessed_from_a_trait_names_the_real_component() {
    let name = UnresolvedWiringName::KeyNamesNoComponent {
        key: "HasScalarTypeComponent".to_owned(),
        consumer: "HasScalarType".to_owned(),
        component: "ScalarTypeProviderComponent".to_owned(),
    };
    assert_eq!(
        plan_unresolved_wiring_name(&name),
        "[CGP-E042] the wiring key `HasScalarTypeComponent` names no component",
    );
    assert_eq!(
        unresolved_wiring_name_help(&name).as_deref(),
        Some("name the component of `HasScalarType` instead: `ScalarTypeProviderComponent`"),
    );
}

#[test]
fn an_undeclared_provider_has_a_help_only_for_a_near_miss() {
    let without = UnresolvedWiringName::ProviderNotDeclared {
        provider: "GreetHi".to_owned(),
        key: "GreeterComponent".to_owned(),
        near_miss: None,
    };
    assert_eq!(
        plan_unresolved_wiring_name(&without),
        "[CGP-E043] the provider `GreetHi` wired for `GreeterComponent` is not declared",
    );
    assert_eq!(unresolved_wiring_name_help(&without), None);
    let with = UnresolvedWiringName::ProviderNotDeclared {
        provider: "GreetHelo".to_owned(),
        key: "GreeterComponent".to_owned(),
        near_miss: Some("GreetHello".to_owned()),
    };
    assert_eq!(
        unresolved_wiring_name_help(&with).as_deref(),
        Some("a provider of `GreeterComponent` with a similar name exists: `GreetHello`"),
    );
}

#[test]
fn a_cgp_impl_provider_not_declared_names_the_new_keyword() {
    let name = UnresolvedWiringName::CgpImplProviderNotDeclared {
        provider: "GreetHello".to_owned(),
    };
    assert_eq!(
        plan_unresolved_wiring_name(&name),
        "[CGP-E038] the provider `GreetHello` is not declared",
    );
    assert_eq!(
        unresolved_wiring_name_help(&name).as_deref(),
        Some("declare it with `#[cgp_impl(new GreetHello)]`"),
    );
}
