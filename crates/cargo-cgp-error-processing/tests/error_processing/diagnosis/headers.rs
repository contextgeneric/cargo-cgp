//! The headline a plan chooses: consumer, provider, and wrapper forms, and which lead it keeps.

use super::*;

/// The lead is dropped by whether the *header* states the leaf, not by the leaf's kind. Under a
/// header that names something else — as the emitter's coalesced block produces, listing the
/// affected consumers — a mismatch keeps its lead, since that is then the only place above the tree
/// where the cause appears.
#[test]
fn keeps_a_mismatch_lead_under_a_header_that_does_not_state_it() {
    for leaf in [
        Leaf::FieldTypeMismatch {
            name: "height".to_owned(),
            owner: "Rectangle".to_owned(),
            expected: "f64".to_owned(),
            expected_normalized: None,
            actual: "i32".to_owned(),
        },
        Leaf::AssocTypeMismatch {
            assoc: "Error".to_owned(),
            trait_name: "HasErrorType".to_owned(),
            owner: "App".to_owned(),
            expected: "AppError".to_owned(),
            expected_normalized: None,
            actual: "String".to_owned(),
            component: Some("ErrorTypeProviderComponent".to_owned()),
        },
    ] {
        let cause = one_path(leaf.clone(), vec![consumer("CanGreet", "App")]);
        let note = &cause_notes(std::slice::from_ref(&cause), None)[0];
        assert!(
            note.starts_with("root cause: ["),
            "expected a kept lead for {leaf:?}, got: {note}"
        );
    }
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
        notes(&plan),
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
    // header reads "the trait" with the `CGP-E003` code, not "the consumer trait".
    let resolved = Resolved {
        context: "MockApp".to_owned(),
        consumers: vec!["CanHandleApiSend<QueryBalanceApi>".to_owned()],
        consumers_are_cgp: false,
        subject_is_context: true,
        causes: [missing_field_cause()].into_iter().collect(),
    };
    assert_eq!(
        consumer_header(&resolved),
        "[CGP-E003] the trait `CanHandleApiSend<QueryBalanceApi>` is not implemented for context `MockApp`"
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
        causes: [missing_field_cause()].into_iter().collect(),
    };
    assert_eq!(
        consumer_header(&resolved),
        "[CGP-E003] the trait `CanAddApiRoutes` is not implemented for `Router<Arc<MockApp>>`"
    );
}
