//! Tests for the wording of a wiring key or value that does not resolve (`[CGP-E038]`,
//! `[CGP-E040]` to `[CGP-E044]`), and of a component a provider header derives that does not
//! (`[CGP-E041]`, `[CGP-E070]`).
//!
//! `plan_unresolved_wiring_name` and `unresolved_wiring_name_help` are pure functions over the
//! rustc-free [`UnresolvedWiringName`] model, driven here over hand-built values.

use cargo_cgp_error_processing::{
    UnresolvedWiringName, plan_unresolved_wiring_name, unresolved_wiring_name_help,
    unresolved_wiring_name_note,
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

#[test]
fn an_undeclared_path_type_has_a_help_only_for_a_whole_segment() {
    let segment = UnresolvedWiringName::PathTypeNotDeclared {
        name: "Missing".to_owned(),
        whole_segment: true,
    };
    assert_eq!(
        plan_unresolved_wiring_name(&segment),
        "[CGP-E044] the type `Missing` in a wiring path is not declared",
    );
    assert_eq!(
        unresolved_wiring_name_help(&segment).as_deref(),
        Some(
            "declare or import the type `Missing`, or write the segment in lowercase to make it a \
             symbol"
        ),
    );
    let nested = UnresolvedWiringName::PathTypeNotDeclared {
        name: "Missing".to_owned(),
        whole_segment: false,
    };
    assert_eq!(
        plan_unresolved_wiring_name(&nested),
        "[CGP-E044] the type `Missing` in a wiring path is not declared",
    );
    assert_eq!(unresolved_wiring_name_help(&nested), None);
}

#[test]
fn a_header_through_a_module_without_the_component_names_the_defining_module() {
    let name = UnresolvedWiringName::ComponentNotInModule {
        component: "GreeterComponent".to_owned(),
        module: "crate::facade".to_owned(),
        trait_path: "crate::components::Greeter".to_owned(),
    };
    assert_eq!(
        plan_unresolved_wiring_name(&name),
        "[CGP-E041] the component `GreeterComponent` is not in module `crate::facade`",
    );
    assert_eq!(
        unresolved_wiring_name_note(&name).as_deref(),
        Some(
            "the header names its provider trait through `crate::facade`, so the macro names the \
             trait's component through that module too"
        ),
    );
    assert_eq!(
        unresolved_wiring_name_help(&name).as_deref(),
        Some(
            "name the trait through the module that defines its component, \
             `impl crate::components::Greeter`, or re-export `GreeterComponent` from \
             `crate::facade` beside the trait"
        ),
    );
}

#[test]
fn a_renamed_import_names_the_real_component_and_the_trait_s_own_name() {
    let name = UnresolvedWiringName::DerivedComponentName {
        derived: "GComponent".to_owned(),
        written: "G".to_owned(),
        provider_trait: "Greeter".to_owned(),
        component: "GreeterComponent".to_owned(),
        attribute: "#[cgp_impl(new GreetHello: GreeterComponent)]".to_owned(),
    };
    assert_eq!(
        plan_unresolved_wiring_name(&name),
        "[CGP-E070] the header derives the component `GComponent` from the trait name `G`, but \
         the component of `Greeter` is `GreeterComponent`",
    );
    assert_eq!(
        unresolved_wiring_name_note(&name).as_deref(),
        Some(
            "`G` imports `Greeter` under another name, so an import of `GreeterComponent` reads \
             as unused while the header names `G`"
        ),
    );
    assert_eq!(
        unresolved_wiring_name_help(&name).as_deref(),
        Some(
            "name the component in the attribute, `#[cgp_impl(new GreetHello: GreeterComponent)]`, \
             or import the trait under its own name, `Greeter`"
        ),
    );
}

#[test]
fn a_renamed_key_names_the_attribute_form_without_a_note() {
    let name = UnresolvedWiringName::DerivedComponentName {
        derived: "AreaCalculatorComponent".to_owned(),
        written: "AreaCalculator".to_owned(),
        provider_trait: "AreaCalculator".to_owned(),
        component: "AreaComponent".to_owned(),
        attribute: "#[cgp_impl(new UnitArea: AreaComponent)]".to_owned(),
    };
    assert_eq!(unresolved_wiring_name_note(&name), None);
    assert_eq!(
        unresolved_wiring_name_help(&name).as_deref(),
        Some("name the component in the attribute: `#[cgp_impl(new UnitArea: AreaComponent)]`"),
    );
}
