//! Tests for the rustc-free diagnosis model and its diagnostic plan.
//!
//! These exercise the wording and planning that used to live inside the driver's emitter, where
//! it could only be pinned end-to-end by a UI snapshot. Moving it into this crate makes each case
//! a plain unit test over a hand-built [`Resolved`], with no compiler in the loop.

use std::collections::HashMap;

use cargo_cgp_error_processing::diagnosis::{mismatch_leaf, quoted_list};
use cargo_cgp_error_processing::rewrite::{ComponentNameMap, ComponentTraitNames};
use cargo_cgp_error_processing::{
    Cause, DependencyTree, DiagKind, FieldIssue, Leaf, Resolved, cause_note, cause_notes,
    cause_signature, consumer_header, dependency_tree_leaf, derive_help_messages,
    field_mismatch_header, plan_resolved, render_dependency_tree, root_cause_code,
};

/// Indent every line by the two spaces the note wording nests a dependency chain under its
/// `this is required through the dependency chain:` heading with.
fn indent2(chain: &str) -> String {
    chain
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

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
        consumers_are_cgp: true,
        subject_is_context: true,
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
        vec![format!(
            "root cause: [CGP-E106] missing field `height` on `Rectangle`\n\
             this is required through the dependency chain:\n{}",
            indent2(&render_dependency_tree(&tree()))
        )]
    );
}

#[test]
fn plans_a_missing_wiring_check_failure() {
    // A two-node spine: the checked consumer over the `CanUseBar` capability the unwired
    // component would supply.
    let tree = DependencyTree::node(
        "consumer trait impl `CanUseFoo` for context `App`",
        vec![DependencyTree::leaf("trait impl `CanUseBar` for `App`")],
    );
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanUseFoo".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![Cause {
            leaf: Leaf::MissingWiring {
                component: "BarProviderComponent".to_owned(),
                owner: "App".to_owned(),
            },
            tree: tree.clone(),
        }],
    };
    let plan = plan_resolved(
        DiagKind::Check,
        Some("the trait bound `App: CanUseComponent<FooProviderComponent>` is not satisfied"),
        &resolved,
        &empty_names(),
    );

    assert_eq!(
        plan.header.as_deref(),
        Some("[CGP-E001] the consumer trait `CanUseFoo` is not implemented for context `App`")
    );
    // A missing wiring, like a genuinely missing field, carries no `help` — the note names the
    // fix.
    assert!(plan.helps.is_empty());
    assert_eq!(
        plan.notes,
        vec![format!(
            "root cause: [CGP-E107] context `App` does not contain any delegate entry for `BarProviderComponent`\n\
             this is required through the dependency chain:\n{}",
            indent2(&render_dependency_tree(&tree))
        )]
    );
}

#[test]
fn plans_a_missing_dispatch_entry_check_failure() {
    // A dispatch table (a `UseInputDelegate` inner table) missing a branch for the type it
    // dispatches on: the checked consumer over the provider stage whose input the table cannot
    // resolve. The leaf names the table and the key, worded as `provider … does not contain any
    // delegate entry for …` — the non-context sibling of a missing wiring, coded `[CGP-E110]`.
    let tree = DependencyTree::node(
        "consumer trait impl `CanHandle<Prog, _>` for context `App`",
        vec![DependencyTree::leaf(
            "provider `SinkHandlers` does not contain any delegate entry for `Tagged<Bytes>`",
        )],
    );
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanHandle<Prog, _>".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![Cause {
            leaf: Leaf::MissingDispatchEntry {
                key: "Tagged<Bytes>".to_owned(),
                table: "SinkHandlers".to_owned(),
            },
            tree: tree.clone(),
        }],
    };
    let plan = plan_resolved(
        DiagKind::MethodNotFound,
        Some("the trait bound `App: CanHandle<Sink, Tagged<Bytes>>` is not satisfied"),
        &resolved,
        &empty_names(),
    );

    // A missing dispatch entry carries no `help`; the note names the fix.
    assert!(plan.helps.is_empty());
    assert_eq!(
        plan.notes,
        vec![format!(
            "root cause: [CGP-E110] provider `SinkHandlers` does not contain any delegate entry for `Tagged<Bytes>`\n\
             this is required through the dependency chain:\n{}",
            indent2(&render_dependency_tree(&tree))
        )]
    );
}

#[test]
fn plans_a_missing_redirect_wiring_check_failure() {
    // A namespace redirect that resolves to nothing: the checked consumer over a redirect-lookup
    // hop whose path the context does not terminate. The redirect node reads as its redirection and
    // the terminal states the missing delegate entry, in the same form a plain missing wiring uses.
    let tree = DependencyTree::node(
        "consumer trait impl `HasQuantityType` for context `App`",
        vec![DependencyTree::node(
            "redirect lookup to `@app.finance.types.QuantityTypeProviderComponent` in `App`",
            vec![DependencyTree::leaf(
                "context `App` does not contain any delegate entry for \
                 `@app.finance.types.QuantityTypeProviderComponent`",
            )],
        )],
    );
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["HasQuantityType".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![Cause {
            leaf: Leaf::MissingRedirectWiring {
                path: "@app.finance.types.QuantityTypeProviderComponent".to_owned(),
                context: "App".to_owned(),
            },
            tree: tree.clone(),
        }],
    };
    let plan = plan_resolved(DiagKind::Check, None, &resolved, &empty_names());

    // A redirect wiring, like a missing wiring, carries no `help` — the note names the fix. Its lead
    // is the same `missing_delegate_entry` phrasing as a plain missing wiring, keyed by the path.
    assert!(plan.helps.is_empty());
    assert_eq!(
        plan.notes,
        vec![format!(
            "root cause: [CGP-E107] context `App` does not contain any delegate entry for \
             `@app.finance.types.QuantityTypeProviderComponent`\n\
             this is required through the dependency chain:\n{}",
            indent2(&render_dependency_tree(&tree))
        )]
    );
}

#[test]
fn plans_a_missing_derive_with_a_help() {
    let resolved = Resolved {
        context: "Rectangle".to_owned(),
        consumers: vec!["CanCalculateArea".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
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
        "root cause: [CGP-E108] accessor trait `HasField` with field `height` is not implemented for `Rectangle`"
    ));
}

#[test]
fn a_deref_target_help_points_at_the_target() {
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanGreet".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
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
        consumers_are_cgp: true,
        subject_is_context: true,
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
            indent2(&render_dependency_tree(&tree()))
        )]
    );
    assert_eq!(mismatch_leaf(&resolved), Some(&mismatch.leaf));
}

#[test]
fn plans_a_use_site_method_failure() {
    let resolved = Resolved {
        context: "Person".to_owned(),
        consumers: vec!["CanGreet".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
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
        consumers_are_cgp: true,
        subject_is_context: true,
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
            indent2(&render_dependency_tree(&tree()))
        )]
    );
}

#[test]
fn promotes_a_mid_chain_symptom_bound_to_the_consumer_header() {
    // rustc opened the diagnostic on a getter bound (`LoginRequest: HasCredential<App>`) that is a
    // symptom, not the recovered root cause (a missing wiring). It is not a recovered leaf, so the
    // header should be replaced with the `CGP-E001` consumer form rather than kept as the symptom.
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanAuthenticate<LoginRequest>".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![Cause {
            leaf: Leaf::MissingWiring {
                component: "CredentialTypeProviderComponent".to_owned(),
                owner: "App".to_owned(),
            },
            tree: tree(),
        }],
    };
    let plan = plan_resolved(
        DiagKind::Check,
        Some("the trait bound `LoginRequest: HasCredential<App>` is not satisfied"),
        &resolved,
        &empty_names(),
    );
    assert_eq!(
        plan.header.as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanAuthenticate<LoginRequest>` is not implemented for context `App`"
        )
    );
}

#[test]
fn dependency_tree_leaf_codes_rewritten_leaves_and_passes_bounds_through() {
    // Each rewritten root-cause leaf carries its own `CGP-E1xx` code as a tree entry…
    assert_eq!(
        dependency_tree_leaf(&Leaf::Field {
            name: "name".to_owned(),
            owner: "App".to_owned(),
            issue: FieldIssue::Missing,
        }),
        "[CGP-E106] missing field `name` on `App`"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::MissingWiring {
            component: "BarProviderComponent".to_owned(),
            owner: "App".to_owned(),
        }),
        "[CGP-E107] context `App` does not contain any delegate entry for `BarProviderComponent`"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::Field {
            name: "name".to_owned(),
            owner: "Person".to_owned(),
            issue: FieldIssue::Present,
        }),
        "[CGP-E108] accessor trait `HasField` with field `name` is not implemented for `Person`"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::FieldTypeMismatch {
            name: "height".to_owned(),
            owner: "Rectangle".to_owned(),
            expected: "f64".to_owned(),
            actual: "i32".to_owned(),
        }),
        "[CGP-E109] field `height` on `Rectangle` has type `i32`, but `f64` is required"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::MissingDispatchEntry {
            key: "Tagged<Bytes>".to_owned(),
            table: "SinkHandlers".to_owned(),
        }),
        "[CGP-E110] provider `SinkHandlers` does not contain any delegate entry for `Tagged<Bytes>`"
    );
    // …but a pass-through ordinary bound keeps rustc's phrasing with no code.
    assert_eq!(
        dependency_tree_leaf(&Leaf::Bound {
            summary: "f64: Eq".to_owned(),
        }),
        "the trait bound `f64: Eq` is not satisfied"
    );

    // The `root cause:` lead reuses the leaf's code, and falls back to the `CGP-E2xx` root-cause
    // code only where the leaf is the uncoded pass-through bound.
    assert_eq!(
        root_cause_code(&Leaf::Field {
            name: "name".to_owned(),
            owner: "App".to_owned(),
            issue: FieldIssue::Missing,
        }),
        "CGP-E106"
    );
    assert_eq!(
        root_cause_code(&Leaf::Bound {
            summary: "f64: Eq".to_owned(),
        }),
        "CGP-E201"
    );
}

#[test]
fn cause_signature_matches_re_reports_and_separates_distinct_failures() {
    let missing_name = || Cause {
        leaf: Leaf::Field {
            name: "name".to_owned(),
            owner: "App".to_owned(),
            issue: FieldIssue::Missing,
        },
        tree: tree(),
    };
    // Same context, consumer, and cause reached at two sites — a re-report — shares a signature,
    // even though the dependency trees differ (the tree is not part of the signature).
    let at_check = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanGreet".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![missing_name()],
    };
    let at_call = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanGreet".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![Cause {
            leaf: Leaf::Field {
                name: "name".to_owned(),
                owner: "App".to_owned(),
                issue: FieldIssue::Missing,
            },
            tree: DependencyTree::leaf("a different chain"),
        }],
    };
    assert_eq!(cause_signature(&at_check), cause_signature(&at_call));

    // A different consumer is a distinct failure — never merged, so no capability is hidden.
    let other_consumer = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanShout".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![missing_name()],
    };
    assert_ne!(cause_signature(&at_check), cause_signature(&other_consumer));

    // A different cause is a distinct fix — never merged.
    let other_cause = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanGreet".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![Cause {
            leaf: Leaf::Field {
                name: "age".to_owned(),
                owner: "App".to_owned(),
                issue: FieldIssue::Missing,
            },
            tree: tree(),
        }],
    };
    assert_ne!(cause_signature(&at_check), cause_signature(&other_cause));
}

#[test]
fn plans_a_provider_header_via_text_rewrite() {
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanFoo".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![Cause {
            leaf: Leaf::Bound {
                summary: "App: DefaultNamespace".to_owned(),
            },
            tree: tree(),
        }],
    };
    // A *real* wired provider (not the `RedirectLookup` plumbing) keeps the provider form, since it
    // names something the programmer chose.
    let plan = plan_resolved(
        DiagKind::Check,
        Some("the trait bound `FooProvider: IsProviderFor<FooComponent, App>` is not satisfied"),
        &resolved,
        &ComponentNameMap::new(foo_names),
    );
    assert_eq!(
        plan.header.as_deref(),
        Some(
            "[CGP-E002] the provider trait `Fooer` with context `App` is not implemented for provider `FooProvider`"
        )
    );
}

#[test]
fn redirect_lookup_provider_follows_through_to_consumer_header() {
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanFoo".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![Cause {
            leaf: Leaf::Bound {
                summary: "App: DefaultNamespace".to_owned(),
            },
            tree: tree(),
        }],
    };
    // An `IsProviderFor` bound whose subject is a `RedirectLookup` names only redirect plumbing —
    // the lookup resolved to no provider — so the header follows through to the consumer trait
    // rather than reporting `RedirectLookup<…>` as the provider. A leading `for<'a>` binder (a
    // higher-ranked obligation) is stripped before the check.
    for subject in [
        "RedirectLookup<App, Nil>",
        "for<'a> RedirectLookup<App, Nil>",
    ] {
        let plan = plan_resolved(
            DiagKind::Check,
            Some(&format!(
                "the trait bound `{subject}: IsProviderFor<FooComponent, App>` is not satisfied"
            )),
            &resolved,
            &ComponentNameMap::new(foo_names),
        );
        assert_eq!(
            plan.header.as_deref(),
            Some("[CGP-E001] the consumer trait `CanFoo` is not implemented for context `App`"),
            "subject `{subject}` should follow through the redirect to the consumer header",
        );
    }
}

#[test]
fn consumer_header_pluralizes_across_components() {
    let resolved = Resolved {
        context: "App".to_owned(),
        consumers: vec!["CanFoo".to_owned(), "CanBar".to_owned()],
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: vec![missing_field_cause()],
    };
    assert_eq!(
        consumer_header(&resolved),
        "[CGP-E001] the consumer traits `CanFoo` and `CanBar` are not implemented for context `App`"
    );
}

#[test]
fn wrapper_header_reads_the_trait_not_the_consumer_trait() {
    // A hand-written wrapper trait (not a CGP consumer) fails through a traced CGP dependency: the
    // header reads "the trait" with the `CGP-E009` code, not "the consumer trait".
    let resolved = Resolved {
        context: "MockApp".to_owned(),
        consumers: vec!["CanHandleApiSend<QueryBalanceApi>".to_owned()],
        consumers_are_cgp: false,
        subject_is_context: true,
        causes: vec![missing_field_cause()],
    };
    assert_eq!(
        consumer_header(&resolved),
        "[CGP-E009] the trait `CanHandleApiSend<QueryBalanceApi>` is not implemented for context `MockApp`"
    );
}

#[test]
fn wrapper_header_names_a_foreign_subject_plainly() {
    // A routing wrapper implemented on a *foreign* type holding the context (`Router<Arc<MockApp>>`)
    // is named plainly — never mislabelled a "context" — while the traced cause on the real context
    // follows in the dependency tree.
    let resolved = Resolved {
        context: "Router<Arc<MockApp>>".to_owned(),
        consumers: vec!["CanAddApiRoutes".to_owned()],
        consumers_are_cgp: false,
        subject_is_context: false,
        causes: vec![missing_field_cause()],
    };
    assert_eq!(
        consumer_header(&resolved),
        "[CGP-E009] the trait `CanAddApiRoutes` is not implemented for `Router<Arc<MockApp>>`"
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
        cause_note(&cause, None)
            .starts_with("root cause: [CGP-E106] missing field `height` on `Rectangle`")
    );
    assert!(derive_help_messages(std::slice::from_ref(&cause)).is_empty());
}

/// One field-missing cause whose chain shares a `getter` node with its sibling — the branch point
/// of the merge test below.
fn field_cause(getter: &str, field: &str) -> Cause {
    Cause {
        leaf: Leaf::Field {
            name: field.to_owned(),
            owner: "Person".to_owned(),
            issue: FieldIssue::Missing,
        },
        tree: DependencyTree::node(
            "consumer trait impl `CanGreet` for context `Person`",
            vec![DependencyTree::node(
                "provider trait impl `Greeter` with context `Person` for provider `GreetFullName`",
                vec![DependencyTree::node(
                    format!("trait impl `{getter}` for `Person`"),
                    vec![DependencyTree::leaf(format!(
                        "[CGP-E106] missing field `{field}` on `Person`"
                    ))],
                )],
            )],
        ),
    }
}

/// Two causes sharing a dependency root collapse into a single `root causes:` note: the shared
/// prefix appears once, each cause is listed up front, and the merged tree branches to the two
/// distinct leaves. A lone cause keeps its own `root cause:` note.
#[test]
fn merges_two_causes_sharing_a_root_into_one_note() {
    let causes = vec![
        field_cause("HasFirstName", "first_name"),
        field_cause("HasLastName", "last_name"),
    ];

    let notes = cause_notes(&causes, None);
    assert_eq!(notes.len(), 1, "the two causes merge into one note");
    assert_eq!(
        notes[0],
        "root causes:\n\
         \x20 - [CGP-E106] missing field `first_name` on `Person`\n\
         \x20 - [CGP-E106] missing field `last_name` on `Person`\n\
         this is required through the dependency chain:\n\
         \x20 consumer trait impl `CanGreet` for context `Person`\n\
         \x20 └─ provider trait impl `Greeter` with context `Person` for provider `GreetFullName`\n\
         \x20   ├─ trait impl `HasFirstName` for `Person`\n\
         \x20   │ └─ [CGP-E106] missing field `first_name` on `Person`\n\
         \x20   └─ trait impl `HasLastName` for `Person`\n\
         \x20     └─ [CGP-E106] missing field `last_name` on `Person`"
    );

    // A single cause is not merged — it keeps the `root cause:` (singular) form.
    let single = cause_notes(std::slice::from_ref(&causes[0]), None);
    assert_eq!(single.len(), 1);
    assert!(single[0].starts_with("root cause: [CGP-E106] missing field `first_name`"));
}
