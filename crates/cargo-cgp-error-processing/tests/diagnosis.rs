//! Tests for the rustc-free diagnosis model and its diagnostic plan.
//!
//! These exercise the wording and planning that used to live inside the driver's emitter, where
//! it could only be pinned end-to-end by a UI snapshot. Moving it into this crate makes each case
//! a plain unit test over a hand-built [`Resolved`], with no compiler in the loop.

use std::collections::HashMap;

use cargo_cgp_error_processing::diagnosis::{mismatch_leaf, quoted_list};
use cargo_cgp_error_processing::rewrite::{ComponentNameMap, ComponentTraitNames};
use cargo_cgp_error_processing::{
    Cause, DependencyTree, DiagKind, FieldIssue, Leaf, Resolved, cause_note, consumer_header,
    derive_help_messages, field_mismatch_header, plan_resolved, render_dependency_tree,
};

/// A two-node dependency spine: the checked consumer over the missing field leaf.
fn tree() -> DependencyTree {
    DependencyTree::node(
        "consumer trait impl `CanCalculateArea` for context `Rectangle`",
        vec![DependencyTree::leaf(
            "field trait impl `HasField` with field `height` for `Rectangle`",
        )],
    )
}

/// An empty name map — the categorized-header paths worded from the resolution never consult it.
fn empty_names() -> ComponentNameMap {
    ComponentNameMap::new(HashMap::new)
}

/// A name map carrying one provider trait, for the `IsProviderFor` text-rewrite header path.
fn foo_names() -> HashMap<String, ComponentTraitNames> {
    let mut map = HashMap::new();
    map.insert(
        "FooComponent".to_owned(),
        ComponentTraitNames {
            consumer: "CanFoo".to_owned(),
            provider: "Fooer".to_owned(),
        },
    );
    map
}

fn missing_field_cause() -> Cause {
    Cause {
        leaf: Leaf::Field {
            name: "height".to_owned(),
            owner: "Rectangle".to_owned(),
            issue: FieldIssue::Missing,
        },
        tree: tree(),
    }
}

#[test]
fn plans_a_missing_field_check_failure() {
    let resolved = Resolved {
        context: "Rectangle".to_owned(),
        consumers: vec!["CanCalculateArea".to_owned()],
        causes: vec![missing_field_cause()],
    };
    let plan = plan_resolved(
        DiagKind::Check,
        Some(
            "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied",
        ),
        &resolved,
        &empty_names(),
    );

    assert_eq!(
        plan.header.as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`"
        )
    );
    assert!(plan.helps.is_empty());
    assert_eq!(
        plan.notes,
        vec![String::from(
            "root cause: missing field `height` on `Rectangle`\n\
             this is required through the dependency chain:\n\
             \x20   consumer trait impl `CanCalculateArea` for context `Rectangle`\n\
             \x20   └── field trait impl `HasField` with field `height` for `Rectangle`"
        )]
    );
}

#[test]
fn plans_a_missing_derive_with_a_help() {
    let resolved = Resolved {
        context: "Rectangle".to_owned(),
        consumers: vec!["CanCalculateArea".to_owned()],
        causes: vec![Cause {
            leaf: Leaf::Field {
                name: "height".to_owned(),
                owner: "Rectangle".to_owned(),
                issue: FieldIssue::Present,
            },
            tree: tree(),
        }],
    };
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
    assert!(plan.notes[0].starts_with(
        "root cause: accessor trait `HasField` with field `height` is not implemented for `Rectangle`"
    ));
}

#[test]
fn a_deref_target_help_points_at_the_target() {
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanGreet".to_owned()],
        causes: vec![Cause {
            leaf: Leaf::Field {
                name: "name".to_owned(),
                owner: "App".to_owned(),
                issue: FieldIssue::PresentViaDeref {
                    target: "AppFields".to_owned(),
                },
            },
            tree: tree(),
        }],
    };
    let plan = plan_resolved(DiagKind::Check, None, &resolved, &empty_names());
    assert_eq!(
        plan.helps,
        vec!["make sure that `#[derive(HasField)]` is used for `AppFields`".to_owned()]
    );
}

#[test]
fn plans_a_field_type_mismatch() {
    let mismatch = Cause {
        leaf: Leaf::FieldTypeMismatch {
            name: "height".to_owned(),
            owner: "Rectangle".to_owned(),
            expected: "f64".to_owned(),
            actual: "i32".to_owned(),
        },
        tree: tree(),
    };
    let resolved = Resolved {
        context: "Rectangle".to_owned(),
        consumers: vec!["CanCalculateArea".to_owned()],
        causes: vec![mismatch.clone()],
    };
    let plan = plan_resolved(
        DiagKind::FieldMismatch,
        Some(
            "type mismatch resolving `<Rectangle as HasField<Symbol!(\"height\")>>::Value == f64`",
        ),
        &resolved,
        &empty_names(),
    );

    assert_eq!(
        plan.header.as_deref(),
        Some("[CGP-E003] expected a `height` field of type `f64` on `Rectangle`, but found `i32`")
    );
    assert!(plan.helps.is_empty());
    // The `[CGP-E003]` header states the leaf, so the note carries the chain alone.
    assert_eq!(
        plan.notes,
        vec![format!(
            "this is required through the dependency chain:\n{}",
            render_dependency_tree(&tree())
        )]
    );
    assert_eq!(mismatch_leaf(&resolved), Some(&mismatch.leaf));
}

#[test]
fn plans_a_use_site_method_failure() {
    let resolved = Resolved {
        context: "Person".to_owned(),
        consumers: vec!["CanGreet".to_owned()],
        causes: vec![Cause {
            leaf: Leaf::Field {
                name: "name".to_owned(),
                owner: "Person".to_owned(),
                issue: FieldIssue::Missing,
            },
            tree: tree(),
        }],
    };
    // A consumer-method `E0599` names no wiring trait, yet the header is still worded from the
    // resolution.
    let plan = plan_resolved(
        DiagKind::MethodNotFound,
        Some("method `greet` not found for this struct"),
        &resolved,
        &empty_names(),
    );
    assert_eq!(
        plan.header.as_deref(),
        Some("[CGP-E001] the consumer trait `CanGreet` is not implemented for context `Person`")
    );
}

#[test]
fn keeps_an_ordinary_bound_header_and_drops_the_repeated_lead() {
    let resolved = Resolved {
        context: "Rectangle".to_owned(),
        consumers: vec!["CanCalculateArea".to_owned()],
        causes: vec![Cause {
            leaf: Leaf::Bound {
                summary: "f64: std::cmp::Eq".to_owned(),
            },
            tree: tree(),
        }],
    };
    let plan = plan_resolved(
        DiagKind::Check,
        Some("the trait bound `f64: std::cmp::Eq` is not satisfied"),
        &resolved,
        &empty_names(),
    );
    // Not a CGP class, so rustc's own header is kept…
    assert_eq!(plan.header, None);
    // …and the note drops its `root cause:` lead, since the kept header already states the bound.
    assert_eq!(
        plan.notes,
        vec![format!(
            "this is required through the dependency chain:\n{}",
            render_dependency_tree(&tree())
        )]
    );
}

#[test]
fn plans_a_provider_header_via_text_rewrite() {
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanFoo".to_owned()],
        causes: vec![Cause {
            leaf: Leaf::Bound {
                summary: "App: DefaultNamespace".to_owned(),
            },
            tree: tree(),
        }],
    };
    let plan = plan_resolved(
        DiagKind::Check,
        Some(
            "the trait bound `RedirectLookup<App, Nil>: IsProviderFor<FooComponent, App>` is not satisfied",
        ),
        &resolved,
        &ComponentNameMap::new(foo_names),
    );
    assert_eq!(
        plan.header.as_deref(),
        Some(
            "[CGP-E002] the provider trait `Fooer` with context `App` is not implemented for provider `RedirectLookup<App, Nil>`"
        )
    );
}

#[test]
fn consumer_header_pluralizes_across_components() {
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanFoo".to_owned(), "CanBar".to_owned()],
        causes: vec![missing_field_cause()],
    };
    assert_eq!(
        consumer_header(&resolved),
        "[CGP-E001] the consumer traits `CanFoo` and `CanBar` are not implemented for context `App`"
    );
}

#[test]
fn wording_helpers_format_directly() {
    assert_eq!(quoted_list(&[]), "");
    assert_eq!(quoted_list(&["a".to_owned()]), "`a`");
    assert_eq!(
        quoted_list(&["a".to_owned(), "b".to_owned()]),
        "`a` and `b`"
    );
    assert_eq!(
        quoted_list(&["a".to_owned(), "b".to_owned(), "c".to_owned()]),
        "`a`, `b`, and `c`"
    );

    assert_eq!(
        field_mismatch_header("height", "Rectangle", "f64", "i32"),
        "[CGP-E003] expected a `height` field of type `f64` on `Rectangle`, but found `i32`"
    );

    let cause = missing_field_cause();
    assert!(
        cause_note(&cause, None).starts_with("root cause: missing field `height` on `Rectangle`")
    );
    assert!(derive_help_messages(std::slice::from_ref(&cause)).is_empty());
}
