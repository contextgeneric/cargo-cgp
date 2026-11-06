//! The headline a plan chooses: consumer, provider, and wrapper forms, and which lead it keeps.

use super::*;

/// The lead is dropped by whether the *header* states the leaf, not by the leaf's kind. Under a
/// header that names something else (as the emitter's coalesced block produces, listing the
/// affected consumers), a mismatch keeps its lead, since that is then the only place above the tree
/// where the cause appears.
// Property: the note keeps its `root cause:` lead whenever the header does not state the leaf, so a
//   mismatch under a header naming something else keeps it.
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

// Property: an ordinary bound keeps rustc's header, and the note drops the lead that header already
//   states (the error-code catalog's `CGP-E117`).
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

// Property: a kept ordinary-bound header writes an empty CGP list as `Product![]`, as its leaf does
//   (the error-code catalog's `CGP-E117`).
#[test]
fn an_ordinary_bound_header_writes_an_empty_list_as_the_leaf_does() {
    // rustc prints CGP's empty list as `Nil`; the leaf, which knows the type, writes `Product![]`.
    // The header still restates the leaf, so it takes the leaf's wording in rustc's sentence,
    // uncoded.
    let leaf = Leaf::Bound {
        summary: "Product![]: PickFirst".to_owned(),
    };
    let cause = one_path(leaf, vec![consumer("CanGreet", "App")]);
    let path = cause.paths[0].clone();
    let resolved = cgp_resolved("App", &["CanGreet"], vec![cause]);
    let plan = plan_resolved(
        DiagKind::Check,
        Some("the trait bound `Nil: PickFirst` is not satisfied"),
        &resolved,
        &empty_names(),
    );
    assert_eq!(
        plan.header.as_deref(),
        Some("the trait bound `Product![]: PickFirst` is not satisfied")
    );
    assert!(!plan.coalesces());
    assert_eq!(
        notes(&plan),
        vec![format!(
            "this is required through the dependency chain:\n{}",
            indent2(&render_path(&path))
        )]
    );
}

// Property: a rustc header naming a bound that is not the recovered leaf is replaced, with the
//   consumer class sentence as the internal headline and the cause as the displayed one (the class
//   code is chosen by the anchor, not rustc's headline).
#[test]
fn promotes_a_mid_chain_symptom_bound_to_the_consumer_header() {
    // rustc opened the diagnostic on a getter bound (`LoginRequest: HasCredential<App>`) that is a
    // symptom, not the recovered root cause (a missing wiring). It is not a recovered leaf, so the
    // header should be replaced with the `CGP-E001` consumer form rather than kept as the symptom.
    let leaf = Leaf::MissingWiring {
        component: "CredentialTypeProviderComponent".to_owned(),
        owner: "App".to_owned(),
        similar: NearMiss::default(),
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
        plan.class_header.as_deref(),
        Some(
            "[CGP-E001] the consumer trait `CanAuthenticate<LoginRequest>` is not implemented for context `App`"
        )
    );
    // The displayed headline states the cause instead.
    assert_eq!(
        plan.header.as_deref(),
        Some("[CGP-E001] missing wiring for `CredentialTypeProviderComponent` on `App`")
    );
}

// Property: the class code follows the anchor's recorded class, never rustc's headline text, so an
//   `IsProviderFor` headline reads as the consumer, provider, or wrapper class its anchor chose
//   (the error-code catalog's `CGP-E002`).
#[test]
fn an_is_provider_for_headline_is_classed_by_its_anchor() {
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
    // rustc's headline names a provider's `IsProviderFor`, but the anchor recovered a consumer, so
    // the class is the consumer's; the provider form the text rewrite gives rustc's headline is
    // kept as a second key for a declined re-report.
    let plan = plan_resolved(
        DiagKind::Check,
        Some("the trait bound `FooProvider: IsProviderFor<FooComponent, App>` is not satisfied"),
        &resolved,
        &ComponentNameMap::new(foo_names),
    );
    assert_eq!(
        plan.class_header.as_deref(),
        Some("[CGP-E001] the consumer trait `CanFoo` is not implemented for context `App`")
    );
    assert_eq!(
        plan.declined_header.as_deref(),
        Some(
            "[CGP-E002] the provider trait `Fooer` with context `App` is not implemented for provider `FooProvider`"
        )
    );
    assert_eq!(
        plan.header.as_deref(),
        Some("[CGP-E001] the trait bound `App: DefaultNamespace` is not satisfied")
    );
}

// Property: a `#[check_providers]` entry is the provider class, naming the provider in the class
//   sentence and stating the cause as the headline (the error-code catalog's `CGP-E002`).
#[test]
fn a_check_providers_entry_is_the_provider_class() {
    let mut resolved = cgp_resolved(
        "Rectangle",
        &["AreaCalculator"],
        vec![one_path(
            field_leaf("scale_factor", "Rectangle", FieldIssue::Missing),
            vec![provider(
                "AreaCalculator",
                "Rectangle",
                "ScaledArea<RectangleArea>",
            )],
        )],
    );
    resolved.class = ResolvedClass::Provider("ScaledArea<RectangleArea>".to_owned());
    let plan = plan_resolved(
        DiagKind::Check,
        Some("the trait bound `Rectangle: HasField<Symbol!(\"scale_factor\")>` is not satisfied"),
        &resolved,
        &empty_names(),
    );
    assert_eq!(
        plan.class_header.as_deref(),
        Some(
            "[CGP-E002] the provider trait `AreaCalculator` with context `Rectangle` is not \
             implemented for provider `ScaledArea<RectangleArea>`"
        )
    );
    assert_eq!(
        plan.header.as_deref(),
        Some("[CGP-E002] missing field `scale_factor` on `Rectangle`")
    );
}

// Property: an `IsProviderFor` bound whose subject is a `RedirectLookup`, with or without a leading
//   binder, reads as the consumer class, never as a provider named `RedirectLookup`.
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
    // An `IsProviderFor` bound whose subject is a `RedirectLookup` names only redirect plumbing,
    // the lookup resolved to no provider, so the header follows through to the consumer trait
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
            plan.class_header.as_deref(),
            Some("[CGP-E001] the consumer trait `CanFoo` is not implemented for context `App`"),
            "subject `{subject}` should follow through the redirect to the consumer header",
        );
    }
}

// Property: a failure across several consumer traits names them all with the plural verb (the
//   error-code catalog's `CGP-E001`).
#[test]
fn consumer_header_pluralizes_across_components() {
    let resolved = cgp_resolved("App", &["CanFoo", "CanBar"], vec![missing_field_cause()]);
    assert_eq!(
        consumer_header(&resolved),
        "[CGP-E001] the consumer traits `CanFoo` and `CanBar` are not implemented for context `App`"
    );
}

// Property: the wrapper class names the trait the program wrote, not a consumer trait (the
//   error-code catalog's `CGP-E003`).
#[test]
fn wrapper_header_reads_the_trait_not_the_consumer_trait() {
    // A hand-written wrapper trait (not a CGP consumer) fails through a traced CGP dependency: the
    // header reads "the trait" with the `CGP-E003` code, not "the consumer trait".
    let resolved = Resolved {
        context: "MockApp".to_owned(),
        consumers: vec!["CanHandleApiSend<QueryBalanceApi>".to_owned()],
        class: ResolvedClass::Wrapper,
        subject_is_context: true,
        causes: [missing_field_cause()].into_iter().collect(),
    };
    assert_eq!(
        consumer_header(&resolved),
        "[CGP-E003] the trait `CanHandleApiSend<QueryBalanceApi>` is not implemented for context `MockApp`"
    );
}

// Property: a wrapper failure on a type without wiring names it plainly, never as a context (the
//   error-code catalog's `CGP-E003`).
#[test]
fn wrapper_header_names_a_foreign_subject_plainly() {
    // A routing wrapper implemented on a *foreign* type holding the context
    // (`Router<Arc<MockApp>>`) is named plainly, never mislabelled a "context", while the traced
    // cause on the real context follows in the dependency tree.
    let resolved = Resolved {
        context: "Router<Arc<MockApp>>".to_owned(),
        consumers: vec!["CanAddApiRoutes".to_owned()],
        class: ResolvedClass::Wrapper,
        subject_is_context: false,
        causes: [missing_field_cause()].into_iter().collect(),
    };
    assert_eq!(
        consumer_header(&resolved),
        "[CGP-E003] the trait `CanAddApiRoutes` is not implemented for `Router<Arc<MockApp>>`"
    );
}
