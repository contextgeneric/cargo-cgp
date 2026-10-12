//! Planning a resolved failure: the header, lead, and note a check or use-site failure becomes.

use super::*;

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
        notes(&plan),
        vec![format!(
            "root cause: [CGP-E110] missing field `height` on `Rectangle`\n\
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
        notes(&plan),
        vec![format!(
            "root cause: [CGP-E113] context `App` does not contain any delegate entry for `BarProviderComponent`\n\
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
        notes(&plan),
        vec![format!(
            "root cause: [CGP-E114] provider `SinkHandlers` does not contain any delegate entry for `Tagged<Bytes>`\n\
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
        notes(&plan),
        vec![format!(
            "root cause: [CGP-E115] the provider trait `ApiHandler` is not implemented for `QueryBalanceRequest`\n\
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
        notes(&plan),
        vec![format!(
            "root cause: [CGP-E113] context `App` does not contain any delegate entry for \
             `@app.finance.types.QuantityTypeProviderComponent`\n\
             this is required through the dependency chain:\n{}",
            indent2(&render_path(&path))
        )]
    );
}

#[test]
fn plans_a_field_type_mismatch() {
    let leaf = Leaf::FieldTypeMismatch {
        name: "height".to_owned(),
        owner: "Rectangle".to_owned(),
        expected: "f64".to_owned(),
        expected_normalized: None,
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
        Some("[CGP-E004] expected a `height` field of type `f64` on `Rectangle`, but found `i32`")
    );
    assert!(plan.helps.is_empty());
    // The `[CGP-E004]` header states the leaf, so the note carries the chain alone.
    assert_eq!(
        notes(&plan),
        vec![format!(
            "this is required through the dependency chain:\n{}",
            indent2(&render_path(&path))
        )]
    );
    assert_eq!(mismatch_leaf(&resolved), Some(&leaf));
}

/// An abstract-type mismatch is the `HasField` mismatch's sibling: the same `E0271` projection
/// failure on a CGP abstract type instead of a field value, so it takes the `[CGP-E005]` header and
/// carries the wiring fix in a `help`.
#[test]
fn plans_an_abstract_type_mismatch() {
    let leaf = Leaf::AssocTypeMismatch {
        assoc: "Error".to_owned(),
        trait_name: "HasErrorType".to_owned(),
        owner: "App".to_owned(),
        expected: "AppError".to_owned(),
        expected_normalized: None,
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
            "[CGP-E005] expected the abstract type `Error` of `HasErrorType` on `App` to be `AppError`, but found `String`"
        )
    );
    assert_eq!(
        plan.helps,
        vec![
            "wire `ErrorTypeProviderComponent` to `UseType<AppError>` in the wiring for `App`, or change the provider to work with `String`"
        ]
    );
    // The `[CGP-E005]` header states the leaf, so the note carries the chain alone.
    assert_eq!(
        notes(&plan),
        vec![format!(
            "this is required through the dependency chain:\n{}",
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
        expected_normalized: None,
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
            "[CGP-E005] expected the associated type `Item` of `Iterator` on `Feed` to be `u8`, but found `u16`"
        )
    );
    assert!(plan.helps.is_empty());
}

#[test]
fn plans_a_use_site_method_failure() {
    let leaf = field_leaf("name", "Person", FieldIssue::Missing);
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
