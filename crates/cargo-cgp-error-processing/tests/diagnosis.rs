//! Tests for the rustc-free diagnosis model and its diagnostic plan.
//!
//! These exercise the wording and planning that used to live inside the driver's emitter, where
//! it could only be pinned end-to-end by a UI snapshot. Moving it into this crate makes each case
//! a plain unit test over a hand-built [`Resolved`], with no compiler in the loop. The dependency
//! chain each note carries is built by the [`DependencyGraph`] from structured node paths; the
//! graph's own rendering is pinned separately in `graph.rs`, so here the focus is the note
//! assembly (lead, heading, indentation, singular vs plural) around it.

use std::collections::HashMap;

use cargo_cgp_error_processing::diagnosis::{assoc_mismatch_leaf, mismatch_leaf, quoted_list};
use cargo_cgp_error_processing::rewrite::{ComponentNameMap, ComponentTraitNames};
use cargo_cgp_error_processing::{
    Cause, ChainNode, DepNode, DependencyGraph, DiagKind, FieldIssue, Leaf, Resolved, cause_note,
    cause_notes, cause_only_signature, cause_signature, consumer_header, dependency_tree_leaf,
    derive_help_messages, field_mismatch_header, plan_resolved, root_cause_code,
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

/// A consumer-trait-impl hop node.
fn consumer(trait_ref: &str, context: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Consumer {
        trait_ref: trait_ref.to_owned(),
        context: context.to_owned(),
    })
}

/// A provider-trait-impl hop node.
fn provider(trait_ref: &str, context: &str, provider: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Provider {
        trait_ref: trait_ref.to_owned(),
        context: context.to_owned(),
        provider: provider.to_owned(),
    })
}

/// A general `trait impl` hop node.
fn trait_hop(trait_ref: &str, self_ty: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Trait {
        trait_ref: trait_ref.to_owned(),
        self_ty: self_ty.to_owned(),
    })
}

/// A redirect-lookup hop node (no dispatched key).
fn redirect(path: &str, context: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Redirect {
        path: path.to_owned(),
        context: context.to_owned(),
        key: String::new(),
    })
}

/// The rendered dependency chain for a single path — the graph over just that path.
fn render_path(path: &[ChainNode]) -> String {
    DependencyGraph::from_paths(&[path.to_vec()]).render()
}

/// A cause with one path down to `leaf` (the terminal node is `leaf` itself).
fn one_path(leaf: Leaf, hops: Vec<ChainNode>) -> Cause {
    let mut path = hops;
    path.push(ChainNode::Leaf(leaf.clone()));
    Cause {
        leaf,
        paths: vec![path],
    }
}

/// A `Resolved` for the common case — CGP consumer traits failing on the context itself, so both
/// `consumers_are_cgp` and `subject_is_context` are `true`. The wrapper-header tests that vary
/// those flags build `Resolved` explicitly instead.
fn cgp_resolved(context: &str, consumers: &[&str], causes: Vec<Cause>) -> Resolved {
    Resolved {
        context: context.to_owned(),
        consumers: consumers.iter().map(|c| (*c).to_owned()).collect(),
        consumers_are_cgp: true,
        subject_is_context: true,
        causes,
    }
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

fn missing_field_leaf() -> Leaf {
    Leaf::Field {
        name: "height".to_owned(),
        owner: "Rectangle".to_owned(),
        issue: FieldIssue::Missing,
    }
}

fn missing_field_cause() -> Cause {
    one_path(
        missing_field_leaf(),
        vec![consumer("CanCalculateArea", "Rectangle")],
    )
}

#[test]
fn plans_a_missing_field_check_failure() {
    let cause = missing_field_cause();
    let path = cause.paths[0].clone();
    let resolved = cgp_resolved("Rectangle", &["CanCalculateArea"], vec![cause]);
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
            indent2(&render_path(&path))
        )]
    );
}

#[test]
fn plans_a_missing_wiring_check_failure() {
    let leaf = Leaf::MissingWiring {
        component: "BarProviderComponent".to_owned(),
        owner: "App".to_owned(),
    };
    let cause = one_path(leaf, vec![consumer("CanUseFoo", "App")]);
    let path = cause.paths[0].clone();
    let resolved = cgp_resolved("App", &["CanUseFoo"], vec![cause]);
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
    // A missing wiring, like a genuinely missing field, carries no `help` — the note names the fix.
    assert!(plan.helps.is_empty());
    assert_eq!(
        plan.notes,
        vec![format!(
            "root cause: [CGP-E107] context `App` does not contain any delegate entry for `BarProviderComponent`\n\
             this is required through the dependency chain:\n{}",
            indent2(&render_path(&path))
        )]
    );
}

#[test]
fn plans_a_missing_dispatch_entry_check_failure() {
    let leaf = Leaf::MissingDispatchEntry {
        key: "Tagged<Bytes>".to_owned(),
        table: "SinkHandlers".to_owned(),
    };
    let cause = one_path(leaf, vec![consumer("CanHandle<Prog, _>", "App")]);
    let path = cause.paths[0].clone();
    let resolved = cgp_resolved("App", &["CanHandle<Prog, _>"], vec![cause]);
    let plan = plan_resolved(
        DiagKind::MethodNotFound,
        Some("the trait bound `App: CanHandle<Sink, Tagged<Bytes>>` is not satisfied"),
        &resolved,
        &empty_names(),
    );

    assert!(plan.helps.is_empty());
    assert_eq!(
        plan.notes,
        vec![format!(
            "root cause: [CGP-E110] provider `SinkHandlers` does not contain any delegate entry for `Tagged<Bytes>`\n\
             this is required through the dependency chain:\n{}",
            indent2(&render_path(&path))
        )]
    );
}

#[test]
fn plans_a_not_a_provider_check_failure() {
    let leaf = Leaf::NotAProvider {
        provider: "QueryBalanceRequest".to_owned(),
        provider_trait: "ApiHandler".to_owned(),
    };
    let cause = one_path(
        leaf,
        vec![consumer("CanHandleApi<QueryBalanceApi>", "MockApp")],
    );
    let path = cause.paths[0].clone();
    let resolved = cgp_resolved("MockApp", &["CanHandleApi<QueryBalanceApi>"], vec![cause]);
    let plan = plan_resolved(DiagKind::Check, None, &resolved, &empty_names());

    assert!(plan.helps.is_empty());
    assert_eq!(
        plan.notes,
        vec![format!(
            "root cause: [CGP-E111] the provider trait `ApiHandler` is not implemented for `QueryBalanceRequest`\n\
             this is required through the dependency chain:\n{}",
            indent2(&render_path(&path))
        )]
    );
}

#[test]
fn plans_a_missing_redirect_wiring_check_failure() {
    let leaf = Leaf::MissingRedirectWiring {
        path: "@app.finance.types.QuantityTypeProviderComponent".to_owned(),
        context: "App".to_owned(),
    };
    let cause = one_path(
        leaf,
        vec![
            consumer("HasQuantityType", "App"),
            redirect("@app.finance.types.QuantityTypeProviderComponent", "App"),
        ],
    );
    let path = cause.paths[0].clone();
    let resolved = cgp_resolved("App", &["HasQuantityType"], vec![cause]);
    let plan = plan_resolved(DiagKind::Check, None, &resolved, &empty_names());

    assert!(plan.helps.is_empty());
    assert_eq!(
        plan.notes,
        vec![format!(
            "root cause: [CGP-E107] context `App` does not contain any delegate entry for \
             `@app.finance.types.QuantityTypeProviderComponent`\n\
             this is required through the dependency chain:\n{}",
            indent2(&render_path(&path))
        )]
    );
}

#[test]
fn plans_a_missing_derive_with_a_help() {
    let leaf = Leaf::Field {
        name: "height".to_owned(),
        owner: "Rectangle".to_owned(),
        issue: FieldIssue::Present,
    };
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
    assert!(plan.notes[0].starts_with(
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

#[test]
fn plans_a_field_type_mismatch() {
    let leaf = Leaf::FieldTypeMismatch {
        name: "height".to_owned(),
        owner: "Rectangle".to_owned(),
        expected: "f64".to_owned(),
        actual: "i32".to_owned(),
    };
    let cause = one_path(
        leaf.clone(),
        vec![consumer("CanCalculateArea", "Rectangle")],
    );
    let path = cause.paths[0].clone();
    let resolved = cgp_resolved("Rectangle", &["CanCalculateArea"], vec![cause]);
    let plan = plan_resolved(
        DiagKind::TypeMismatch,
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
            indent2(&render_path(&path))
        )]
    );
    assert_eq!(mismatch_leaf(&resolved), Some(&leaf));
}

/// An abstract-type mismatch is the `HasField` mismatch's sibling: the same `E0271` projection
/// failure on a CGP abstract type instead of a field value, so it takes the `[CGP-E017]` header and
/// carries the wiring fix in a `help`.
#[test]
fn plans_an_abstract_type_mismatch() {
    let leaf = Leaf::AssocTypeMismatch {
        assoc: "Error".to_owned(),
        trait_name: "HasErrorType".to_owned(),
        owner: "App".to_owned(),
        expected: "AppError".to_owned(),
        actual: "String".to_owned(),
        component: Some("ErrorTypeProviderComponent".to_owned()),
    };
    let cause = one_path(leaf.clone(), vec![consumer("CanRaiseHttpError", "App")]);
    let path = cause.paths[0].clone();
    let resolved = cgp_resolved("App", &["CanRaiseHttpError"], vec![cause]);
    let plan = plan_resolved(
        DiagKind::TypeMismatch,
        Some("type mismatch resolving `<App as HasErrorType>::Error == AppError`"),
        &resolved,
        &empty_names(),
    );

    assert_eq!(
        plan.header.as_deref(),
        Some(
            "[CGP-E017] expected the abstract type `Error` of `HasErrorType` on `App` to be `AppError`, but found `String`"
        )
    );
    assert_eq!(
        plan.helps,
        vec![
            "wire `ErrorTypeProviderComponent` to `UseType<AppError>` in the wiring for `App`, or change the provider to work with `String`"
        ]
    );
    assert_eq!(
        plan.notes,
        vec![format!(
            "root cause: [CGP-E112] abstract type `Error` of `HasErrorType` on `App` is `String`, but `AppError` is required\nthis is required through the dependency chain:\n{}",
            indent2(&render_path(&path))
        )]
    );
    assert_eq!(assoc_mismatch_leaf(&resolved), Some(&leaf));
}

/// An associated type on a trait that is *not* a CGP abstract-type component reads as an
/// `associated type` and carries no `help`: there is no wiring entry to name, since the type is
/// fixed by whatever impl supplies it.
#[test]
fn plans_a_plain_associated_type_mismatch() {
    let leaf = Leaf::AssocTypeMismatch {
        assoc: "Item".to_owned(),
        trait_name: "Iterator".to_owned(),
        owner: "Feed".to_owned(),
        expected: "u8".to_owned(),
        actual: "u16".to_owned(),
        component: None,
    };
    let resolved = cgp_resolved(
        "App",
        &["CanRead"],
        vec![one_path(leaf, vec![consumer("CanRead", "App")])],
    );
    let plan = plan_resolved(
        DiagKind::TypeMismatch,
        Some("type mismatch resolving `<Feed as Iterator>::Item == u8`"),
        &resolved,
        &empty_names(),
    );

    assert_eq!(
        plan.header.as_deref(),
        Some(
            "[CGP-E017] expected the associated type `Item` of `Iterator` on `Feed` to be `u8`, but found `u16`"
        )
    );
    assert!(plan.helps.is_empty());
}

#[test]
fn plans_a_use_site_method_failure() {
    let leaf = Leaf::Field {
        name: "name".to_owned(),
        owner: "Person".to_owned(),
        issue: FieldIssue::Missing,
    };
    let resolved = cgp_resolved(
        "Person",
        &["CanGreet"],
        vec![one_path(leaf, vec![consumer("CanGreet", "Person")])],
    );
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
    let leaf = Leaf::Bound {
        summary: "f64: std::cmp::Eq".to_owned(),
    };
    let cause = one_path(leaf, vec![consumer("CanCalculateArea", "Rectangle")]);
    let path = cause.paths[0].clone();
    let resolved = cgp_resolved("Rectangle", &["CanCalculateArea"], vec![cause]);
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
            indent2(&render_path(&path))
        )]
    );
}

#[test]
fn promotes_a_mid_chain_symptom_bound_to_the_consumer_header() {
    // rustc opened the diagnostic on a getter bound (`LoginRequest: HasCredential<App>`) that is a
    // symptom, not the recovered root cause (a missing wiring). It is not a recovered leaf, so the
    // header should be replaced with the `CGP-E001` consumer form rather than kept as the symptom.
    let leaf = Leaf::MissingWiring {
        component: "CredentialTypeProviderComponent".to_owned(),
        owner: "App".to_owned(),
    };
    let resolved = cgp_resolved(
        "App",
        &["CanAuthenticate<LoginRequest>"],
        vec![one_path(
            leaf,
            vec![consumer("CanAuthenticate<LoginRequest>", "App")],
        )],
    );
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
    assert_eq!(
        dependency_tree_leaf(&Leaf::NotAProvider {
            provider: "QueryBalanceRequest".to_owned(),
            provider_trait: "ApiHandler".to_owned(),
        }),
        "[CGP-E111] the provider trait `ApiHandler` is not implemented for `QueryBalanceRequest`"
    );
    assert_eq!(
        dependency_tree_leaf(&Leaf::AssocTypeMismatch {
            assoc: "Error".to_owned(),
            trait_name: "HasErrorType".to_owned(),
            owner: "App".to_owned(),
            expected: "AppError".to_owned(),
            actual: "String".to_owned(),
            component: Some("ErrorTypeProviderComponent".to_owned()),
        }),
        "[CGP-E112] abstract type `Error` of `HasErrorType` on `App` is `String`, but `AppError` is required"
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
    let missing_name = || {
        one_path(
            Leaf::Field {
                name: "name".to_owned(),
                owner: "App".to_owned(),
                issue: FieldIssue::Missing,
            },
            vec![consumer("CanGreet", "App")],
        )
    };
    // Same context, consumer, and cause reached at two sites — a re-report — shares a signature,
    // even though the dependency paths differ (the path is not part of the signature).
    let at_check = cgp_resolved("App", &["CanGreet"], vec![missing_name()]);
    let at_call = cgp_resolved(
        "App",
        &["CanGreet"],
        vec![one_path(
            Leaf::Field {
                name: "name".to_owned(),
                owner: "App".to_owned(),
                issue: FieldIssue::Missing,
            },
            vec![consumer("CanGreet", "App"), trait_hop("Different", "App")],
        )],
    );
    assert_eq!(cause_signature(&at_check), cause_signature(&at_call));

    // A different consumer is a distinct failure — never merged, so no capability is hidden.
    let other_consumer = cgp_resolved("App", &["CanShout"], vec![missing_name()]);
    assert_ne!(cause_signature(&at_check), cause_signature(&other_consumer));

    // A different cause is a distinct fix — never merged.
    let other_cause = cgp_resolved(
        "App",
        &["CanGreet"],
        vec![one_path(
            Leaf::Field {
                name: "age".to_owned(),
                owner: "App".to_owned(),
                issue: FieldIssue::Missing,
            },
            vec![consumer("CanGreet", "App")],
        )],
    );
    assert_ne!(cause_signature(&at_check), cause_signature(&other_cause));

    // The cause-only signature drops the consumer, so two *different* consumers that share one
    // cause group together (for coalescing) while a different cause still separates them.
    assert_eq!(
        cause_only_signature(&at_check),
        cause_only_signature(&other_consumer),
    );
    assert_ne!(
        cause_only_signature(&at_check),
        cause_only_signature(&other_cause),
    );
}

#[test]
fn plans_a_provider_header_via_text_rewrite() {
    let resolved = cgp_resolved(
        "App",
        &["CanFoo"],
        vec![one_path(
            Leaf::Bound {
                summary: "App: DefaultNamespace".to_owned(),
            },
            vec![consumer("CanFoo", "App")],
        )],
    );
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
    let resolved = cgp_resolved(
        "App",
        &["CanFoo"],
        vec![one_path(
            Leaf::Bound {
                summary: "App: DefaultNamespace".to_owned(),
            },
            vec![consumer("CanFoo", "App")],
        )],
    );
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
    let resolved = cgp_resolved("App", &["CanFoo", "CanBar"], vec![missing_field_cause()]);
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

/// One field-missing cause whose path shares a `getter` hop's prefix with its sibling — the branch
/// point of the merge test below.
fn field_cause(getter: &str, field: &str) -> Cause {
    one_path(
        Leaf::Field {
            name: field.to_owned(),
            owner: "Person".to_owned(),
            issue: FieldIssue::Missing,
        },
        vec![
            consumer("CanGreet", "Person"),
            provider("Greeter", "Person", "GreetFullName"),
            trait_hop(getter, "Person"),
        ],
    )
}

/// Two causes sharing a dependency prefix collapse into a single `root causes:` note: the shared
/// prefix appears once, each cause is listed up front, and the merged graph branches to the two
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
         \x20 [CGP-E101] consumer trait impl `CanGreet` for context `Person`\n\
         \x20 └─ [CGP-E102] provider trait impl `Greeter` with context `Person` for provider `GreetFullName`\n\
         \x20   ├─ [CGP-E105] trait impl `HasFirstName` for `Person`\n\
         \x20   │ └─ [CGP-E106] missing field `first_name` on `Person`\n\
         \x20   └─ [CGP-E105] trait impl `HasLastName` for `Person`\n\
         \x20     └─ [CGP-E106] missing field `last_name` on `Person`"
    );

    // A single cause is not merged — it keeps the `root cause:` (singular) form.
    let single = cause_notes(std::slice::from_ref(&causes[0]), None);
    assert_eq!(single.len(), 1);
    assert!(single[0].starts_with("root cause: [CGP-E106] missing field `first_name`"));
}

/// Two causes reached through *independent* consumer chains that share no node still collapse into
/// one note (the `parallel_consumers` shape), not two: the `root causes:` heading lists each distinct
/// leaf, and the graph renders each root chain stacked. This is the behavior the graph adds over the
/// old shared-prefix-only merge, which would have emitted two separate notes here.
#[test]
fn independent_root_causes_render_as_one_note_with_stacked_chains() {
    let causes = vec![
        one_path(
            Leaf::Field {
                name: "height".to_owned(),
                owner: "App".to_owned(),
                issue: FieldIssue::Missing,
            },
            vec![consumer("CanCalculateArea", "App")],
        ),
        one_path(
            Leaf::Field {
                name: "width".to_owned(),
                owner: "App".to_owned(),
                issue: FieldIssue::Missing,
            },
            vec![consumer("CanReportWidth", "App")],
        ),
    ];

    let notes = cause_notes(&causes, None);
    assert_eq!(
        notes.len(),
        1,
        "independent causes still merge into one note"
    );
    assert_eq!(
        notes[0],
        "root causes:\n\
         \x20 - [CGP-E106] missing field `height` on `App`\n\
         \x20 - [CGP-E106] missing field `width` on `App`\n\
         this is required through the dependency chain:\n\
         \x20 [CGP-E101] consumer trait impl `CanCalculateArea` for context `App`\n\
         \x20 └─ [CGP-E106] missing field `height` on `App`\n\
         \x20 [CGP-E101] consumer trait impl `CanReportWidth` for context `App`\n\
         \x20 └─ [CGP-E106] missing field `width` on `App`"
    );
}
