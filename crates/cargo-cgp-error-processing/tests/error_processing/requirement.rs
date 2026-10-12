//! Tests for the undeclared-requirement wording (`[CGP-E036]`).
//!
//! `plan_undeclared_requirement` and `undeclared_requirement_helps` are pure functions over the
//! rustc-free [`UndeclaredRequirement`] model, driven here over hand-built values. The driver fills
//! the model in when a `#[cgp_impl]` provider omits a bound its component requires of the context.

use cargo_cgp_error_processing::{
    Requirement, UndeclaredRequirement, plan_undeclared_requirement, undeclared_requirement_helps,
};

fn trait_bound(bound: &str) -> Requirement {
    Requirement {
        bound: bound.to_owned(),
        import: None,
    }
}

fn abstract_type(bound: &str, import: &str) -> Requirement {
    Requirement {
        bound: bound.to_owned(),
        import: Some(import.to_owned()),
    }
}

fn undeclared(requirements: Vec<Requirement>) -> UndeclaredRequirement {
    UndeclaredRequirement {
        provider_trait: "Greeter".to_owned(),
        provider: "GreetHello".to_owned(),
        requirements,
    }
}

#[test]
fn one_requirement_is_named_with_its_declaration() {
    let model = undeclared(vec![trait_bound("HasName")]);
    assert_eq!(
        plan_undeclared_requirement(&model),
        "[CGP-E036] the provider trait `Greeter` requires `HasName` of the context, but the \
         provider `GreetHello` does not declare it",
    );
    assert_eq!(
        undeclared_requirement_helps(&model),
        vec!["declare it as a dependency with `#[uses(HasName)]`".to_owned()],
    );
}

#[test]
fn several_requirements_are_declared_together() {
    let model = undeclared(vec![trait_bound("HasName"), trait_bound("TagFor<u32>")]);
    assert_eq!(
        plan_undeclared_requirement(&model),
        "[CGP-E036] the provider trait `Greeter` requires `HasName` and `TagFor<u32>` of the \
         context, but the provider `GreetHello` does not declare them",
    );
    assert_eq!(
        undeclared_requirement_helps(&model),
        vec!["declare them as dependencies with `#[uses(HasName, TagFor<u32>)]`".to_owned()],
    );
}

#[test]
fn an_abstract_type_is_imported_rather_than_declared() {
    let model = undeclared(vec![
        trait_bound("HasName"),
        abstract_type("HasErrorType", "HasErrorType.Error"),
    ]);
    assert_eq!(
        undeclared_requirement_helps(&model),
        vec![
            "declare it as a dependency with `#[uses(HasName)]`".to_owned(),
            "import it with `#[use_type(HasErrorType.Error)]`".to_owned(),
        ],
    );
    let two = undeclared(vec![
        abstract_type("HasErrorType", "HasErrorType.Error"),
        abstract_type("HasScalarType", "HasScalarType.Scalar"),
    ]);
    assert_eq!(
        undeclared_requirement_helps(&two),
        vec!["import them with `#[use_type(HasErrorType.Error, HasScalarType.Scalar)]`".to_owned()],
    );
}
