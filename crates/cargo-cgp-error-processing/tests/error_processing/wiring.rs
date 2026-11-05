//! Tests for the duplicate-key wiring-conflict wording.
//!
//! `plan_wiring_conflict` is a pure function over the rustc-free [`WiringConflict`] model, so it
//! is driven directly over hand-built conflicts, with no compiler, no diagnostic wrapper. The driver
//! fills the same model in from the live `TyCtxt`. Each shape carries its own `[CGP-E0xx]` code,
//! and an `@`-path renders in bare notation (no `Path!(…)` wrapper).

use cargo_cgp_error_processing::{
    WiringConflict, WiringKey, plan_wiring_conflict, wiring_conflict_help,
};

#[test]
fn duplicate_component_key() {
    let conflict = WiringConflict::Duplicate {
        context: "Person".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E010] conflicting wiring: `Person` wires component `GreeterComponent` twice",
    );
}

#[test]
fn duplicate_path_key() {
    let conflict = WiringConflict::Duplicate {
        context: "App".to_owned(),
        key: WiringKey::Path("@cgp.core.error.ErrorTypeProviderComponent".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E010] conflicting wiring: `App` wires `@cgp.core.error.ErrorTypeProviderComponent` twice",
    );
}

#[test]
fn duplicate_type_key() {
    // A per-type default's key names a type, not a component, so it is worded as one.
    let conflict = WiringConflict::Duplicate {
        context: "DefaultImpls1<ShowImplComponent>".to_owned(),
        key: WiringKey::Type("String".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E010] conflicting wiring: `DefaultImpls1<ShowImplComponent>` wires type `String` twice",
    );
}

#[test]
fn overlap_bare_component_over_namespace() {
    let conflict = WiringConflict::Overlap {
        context: "App".to_owned(),
        conflicting: WiringKey::Component("ErrorTypeProviderComponent".to_owned()),
        first: WiringKey::Blanket("DefaultNamespace".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E011] conflicting wiring: `App` wires component `ErrorTypeProviderComponent`, which `DefaultNamespace` already covers",
    );
}

#[test]
fn overlap_path_over_namespace() {
    let conflict = WiringConflict::Overlap {
        context: "App".to_owned(),
        conflicting: WiringKey::Path("@app.GreeterComponent".to_owned()),
        first: WiringKey::Blanket("AppNamespace".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E011] conflicting wiring: `App` wires `@app.GreeterComponent`, which `AppNamespace` already covers",
    );
}

#[test]
fn overlap_path_prefix_of_path() {
    let conflict = WiringConflict::Overlap {
        context: "App".to_owned(),
        conflicting: WiringKey::Path(
            "@cgp.core.error.ErrorTypeProviderComponent.String".to_owned(),
        ),
        first: WiringKey::Path("@cgp.core.error.ErrorTypeProviderComponent".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E011] conflicting wiring: `App` wires `@cgp.core.error.ErrorTypeProviderComponent.String`, which `@cgp.core.error.ErrorTypeProviderComponent` already covers",
    );
}

#[test]
fn multiple_namespaces() {
    // A namespace join plus a bare-key `for` loop, or two namespace joins: two blanket forwardings
    // over every key. `namespace` desugars to a bare-key `for`, so the two shapes coincide.
    let conflict = WiringConflict::MultipleNamespaces {
        context: "App".to_owned(),
        first: "DefaultNamespace".to_owned(),
        second: "GreeterTable".to_owned(),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E012] conflicting wiring: `App` joins both `DefaultNamespace` and `GreeterTable`, but a target type in `delegate_components!` can join only one namespace",
    );
}

// Property: a redirect collision names its subject, the key it wires directly, and the path that
//   redirects it, with the fix in a `help` naming the written provider at that path (the error-code
//   catalog's `CGP-E013`). The subject may be a context or a namespace binding a key its parent
//   redirects; the wording is the same for both.
#[test]
fn redirect_collides_with_direct_wiring() {
    // The fix moves to a `help`, keeping the header a single short sentence.
    let conflict = WiringConflict::Redirect {
        context: "Person".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
        path: "@GreeterComponent".to_owned(),
        provider: Some("GreetHello".to_owned()),
        params: Vec::new(),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E013] conflicting wiring: `Person` wires component `GreeterComponent` directly, but it is redirected to `@GreeterComponent`",
    );
    assert_eq!(
        wiring_conflict_help(&conflict).as_deref(),
        Some("wire the provider `GreetHello` with the key `@GreeterComponent`"),
    );
}

#[test]
fn redirect_of_generic_component_names_a_generic_key() {
    // The redirect appends the component's parameters to the path, so wiring the provider at the
    // bare path would never be looked up; a generic segment per parameter serves every value the
    // direct entry served.
    let conflict = WiringConflict::Redirect {
        context: "App".to_owned(),
        key: WiringKey::Component("AreaCalculatorComponent".to_owned()),
        path: "@AreaCalculatorComponent".to_owned(),
        provider: Some("UseDelegate<AreaTable>".to_owned()),
        params: vec!["Shape".to_owned()],
    };
    assert_eq!(
        wiring_conflict_help(&conflict).as_deref(),
        Some(
            "wire the provider `UseDelegate<AreaTable>` with the key \
             `@AreaCalculatorComponent.<Shape> Shape`"
        ),
    );

    let conflict = WiringConflict::Redirect {
        context: "App".to_owned(),
        key: WiringKey::Component("PairComponent".to_owned()),
        path: "@PairComponent".to_owned(),
        provider: Some("PairProvider".to_owned()),
        params: vec!["A".to_owned(), "B".to_owned()],
    };
    assert_eq!(
        wiring_conflict_help(&conflict).as_deref(),
        Some("wire the provider `PairProvider` with the key `@PairComponent.<A, B> A.B`"),
    );

    // With no provider a reader wrote, there is nothing to wire under the key.
    let conflict = WiringConflict::Redirect {
        context: "App".to_owned(),
        key: WiringKey::Component("PairComponent".to_owned()),
        path: "@PairComponent".to_owned(),
        provider: None,
        params: vec!["A".to_owned(), "B".to_owned()],
    };
    assert_eq!(wiring_conflict_help(&conflict), None);
}

#[test]
fn redirect_without_a_written_provider_has_no_help() {
    let conflict = WiringConflict::Redirect {
        context: "App".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
        path: "@GreeterComponent".to_owned(),
        provider: None,
        params: Vec::new(),
    };
    assert_eq!(wiring_conflict_help(&conflict), None);
}

#[test]
fn non_redirect_conflicts_have_no_help() {
    let conflict = WiringConflict::Duplicate {
        context: "Person".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
    };
    assert_eq!(wiring_conflict_help(&conflict), None);
}

#[test]
fn duplicate_redirect_to_different_paths() {
    let conflict = WiringConflict::DuplicateRedirect {
        context: "App".to_owned(),
        key: WiringKey::Component("FooComponent".to_owned()),
        first_path: "@app.foo".to_owned(),
        second_path: "@app.bar".to_owned(),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E014] conflicting wiring: `App` redirects component `FooComponent` twice, to `@app.foo` and to `@app.bar`",
    );
}

#[test]
fn duplicate_redirect_to_same_path() {
    let conflict = WiringConflict::DuplicateRedirect {
        context: "Person".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
        first_path: "@GreeterComponent".to_owned(),
        second_path: "@GreeterComponent".to_owned(),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E014] conflicting wiring: `Person` redirects component `GreeterComponent` twice, both times to `@GreeterComponent`",
    );
}
