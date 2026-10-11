//! The `help` lines a plan attaches, such as the missing-derive help.

use super::*;

#[test]
fn plans_a_missing_derive_with_a_help() {
    let leaf = field_leaf("height", "Rectangle", FieldIssue::Present);
    let resolved = cgp_resolved(
        "Rectangle",
        &["CanCalculateArea"],
        vec![one_path(
            leaf,
            vec![consumer("CanCalculateArea", "Rectangle")],
        )],
    );
    let plan = plan_resolved(
        DiagKind::Check,
        Some(
            "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied",
        ),
        &resolved,
        &empty_names(),
    );

    assert_eq!(
        plan.helps,
        vec!["make sure that `#[derive(HasField)]` is used for `Rectangle`".to_owned()]
    );
    assert!(notes(&plan)[0].starts_with(
        "root cause: [CGP-E108] accessor trait `HasField` with field `height` is not implemented for `Rectangle`"
    ));
}

#[test]
fn a_deref_target_help_points_at_the_target() {
    let leaf = Leaf::Field {
        name: "name".to_owned(),
        owner: "App".to_owned(),
        issue: FieldIssue::PresentViaDeref {
            target: "AppFields".to_owned(),
        },
    };
    let resolved = cgp_resolved(
        "App",
        &["CanGreet"],
        vec![one_path(leaf, vec![consumer("CanGreet", "App")])],
    );
    let plan = plan_resolved(DiagKind::Check, None, &resolved, &empty_names());
    assert_eq!(
        plan.helps,
        vec!["make sure that `#[derive(HasField)]` is used for `AppFields`".to_owned()]
    );
}
