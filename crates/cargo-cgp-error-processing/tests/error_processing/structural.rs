//! Tests for the structural-mistake wording.
//!
//! `plan_structural_mistake` and `structural_mistake_help` are pure functions over the rustc-free
//! [`StructuralMistake`] model, so they are driven over hand-built mistakes; the driver fills the
//! same model in from the generated items an error lands on.

use cargo_cgp_error_processing::{
    CheckForm, CheckTraitName, EntryTable, ParameterUse, StructuralMistake,
    plan_structural_mistake, structural_mistake_help,
};

#[test]
fn a_duplicate_check_names_the_entry_and_the_context() {
    let mistake = StructuralMistake::DuplicateCheck {
        context: "App".to_owned(),
        entry: "GreeterComponent".to_owned(),
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E015] `check_components!` checks `GreeterComponent` twice for `App`",
    );
    assert_eq!(
        structural_mistake_help(&mistake),
        "remove one of the two `GreeterComponent` entries",
    );
}

#[test]
fn a_duplicate_check_names_a_generic_entry_as_the_table_wrote_it() {
    let mistake = StructuralMistake::DuplicateCheck {
        context: "App".to_owned(),
        entry: "ShowerComponent: u32".to_owned(),
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E015] `check_components!` checks `ShowerComponent: u32` twice for `App`",
    );
}

#[test]
fn two_tables_with_a_derived_check_trait_name_the_context() {
    let mistake = StructuralMistake::DuplicateCheckTrait {
        name: CheckTraitName::Derived {
            context: "App".to_owned(),
        },
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E016] two `check_components!` tables in one module check `App`",
    );
    assert_eq!(
        structural_mistake_help(&mistake),
        "merge the two tables, or name one table's check trait with `#[check_trait(...)]`",
    );
}

#[test]
fn two_tables_with_a_given_check_trait_name_it() {
    let mistake = StructuralMistake::DuplicateCheckTrait {
        name: CheckTraitName::Given {
            check_trait: "CheckApp".to_owned(),
        },
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E016] two `check_components!` tables in one module name their check trait `CheckApp`",
    );
    assert_eq!(
        structural_mistake_help(&mistake),
        "merge the two tables, or give one of them another `#[check_trait(...)]` name",
    );
}

#[test]
fn a_loop_key_without_its_variable_names_both() {
    let mistake = StructuralMistake::LoopKeyWithoutVariable {
        context: "App".to_owned(),
        variable: "T".to_owned(),
        key: "@test.ShowImplComponent.String".to_owned(),
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E017] the `for` loop in the wiring of `App` binds `T`, but its key `@test.ShowImplComponent.String` does not use it",
    );
    assert_eq!(
        structural_mistake_help(&mistake),
        "write `T` in the key where the type the loop covers goes",
    );
}

#[test]
fn a_default_impl_on_a_generic_impl_names_the_parameter() {
    let mistake = StructuralMistake::GenericDefaultImpl {
        provider: "ShowWithDisplay".to_owned(),
        key: "String".to_owned(),
        parameter: "T".to_owned(),
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E018] `#[default_impl]` cannot register `ShowWithDisplay` for `String`, because the provider's impl is generic over `T`",
    );
    assert_eq!(
        structural_mistake_help(&mistake),
        "write `ShowWithDisplay`'s impl for `String` alone, in the concise `#[cgp_impl]` form without `T`",
    );
}

#[test]
fn an_entry_parameter_outside_the_key_names_the_entry_and_its_provider() {
    let mistake = StructuralMistake::EntryParametersNotInKey {
        table: EntryTable::Context("Person".to_owned()),
        key: "GreeterComponent".to_owned(),
        parameters: vec!["T".to_owned()],
        used: ParameterUse::Provider("GreetWith<T>".to_owned()),
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E019] the entry for `GreeterComponent` in the wiring of `Person` binds `T`, but its key does not use it",
    );
    assert_eq!(
        structural_mistake_help(&mistake),
        "write a concrete type in place of `T` in `GreetWith<T>`, and remove `T` from the entry's generic parameters",
    );
}

#[test]
fn several_entry_parameters_are_named_together() {
    let mistake = StructuralMistake::EntryParametersNotInKey {
        table: EntryTable::Context("Person".to_owned()),
        key: "GreeterComponent".to_owned(),
        parameters: vec!["T".to_owned(), "U".to_owned()],
        used: ParameterUse::Provider("GreetPair<T, U>".to_owned()),
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E019] the entry for `GreeterComponent` in the wiring of `Person` binds `T` and `U`, but its key does not use them",
    );
    assert_eq!(
        structural_mistake_help(&mistake),
        "write concrete types in place of `T` and `U` in `GreetPair<T, U>`, and remove `T` and `U` from the entry's generic parameters",
    );
}

#[test]
fn an_entry_parameter_used_outside_a_provider_points_at_its_uses() {
    let mistake = StructuralMistake::EntryParametersNotInKey {
        table: EntryTable::Context("Kiosk".to_owned()),
        key: "GreeterComponent".to_owned(),
        parameters: vec!["T".to_owned()],
        used: ParameterUse::Value,
    };
    assert_eq!(
        structural_mistake_help(&mistake),
        "write a concrete type in place of `T` where the entry uses it, and remove `T` from the entry's generic parameters",
    );
}

#[test]
fn an_unused_entry_parameter_is_only_removed() {
    let mistake = StructuralMistake::EntryParametersNotInKey {
        table: EntryTable::Context("Robot".to_owned()),
        key: "GreeterComponent".to_owned(),
        parameters: vec!["T".to_owned()],
        used: ParameterUse::Unused,
    };
    assert_eq!(
        structural_mistake_help(&mistake),
        "remove `T` from the entry's generic parameters",
    );
}

#[test]
fn a_namespace_entry_names_the_namespace() {
    let mistake = StructuralMistake::EntryParametersNotInKey {
        table: EntryTable::Namespace("AppNamespace".to_owned()),
        key: "@app.GreeterComponent".to_owned(),
        parameters: vec!["T".to_owned()],
        used: ParameterUse::Provider("GreetWith<T>".to_owned()),
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E019] the entry for `@app.GreeterComponent` in the namespace `AppNamespace` binds `T`, but its key does not use it",
    );
}

#[test]
fn a_check_at_unit_parameters_names_the_parameters_to_check() {
    let mistake = |form, parameters: &[&str]| StructuralMistake::CheckAtUnitParameters {
        context: "App".to_owned(),
        entry: "AreaCalculatorComponent".to_owned(),
        parameters: parameters.iter().map(|&p| p.to_owned()).collect(),
        form,
    };
    let delegate_and_check = mistake(CheckForm::DelegateAndCheckComponents, &["Rectangle"]);
    assert_eq!(
        plan_structural_mistake(&delegate_and_check),
        "[CGP-E060] the check of `AreaCalculatorComponent` on `App` names no parameters, so it \
         checks the component at the unit parameters `()`",
    );
    assert_eq!(
        structural_mistake_help(&delegate_and_check),
        "check it at the parameters its provider implements: `#[check_params(Rectangle)]`",
    );
    assert_eq!(
        structural_mistake_help(&mistake(CheckForm::CheckComponents, &["Rectangle"])),
        "check it at the parameters its provider implements: `AreaCalculatorComponent: Rectangle`",
    );
    assert_eq!(
        structural_mistake_help(&mistake(
            CheckForm::CheckComponents,
            &["Rectangle", "Circle"]
        )),
        "check it at the parameters its provider implements: `AreaCalculatorComponent: [Rectangle, Circle]`",
    );
    assert_eq!(
        structural_mistake_help(&mistake(
            CheckForm::DelegateAndCheckComponents,
            &["Rectangle", "Circle"],
        )),
        "check it at the parameters its provider implements: `#[check_params(Rectangle, Circle)]`",
    );
}
