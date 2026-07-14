//! Tests for the duplicate-key wiring-conflict wording.
//!
//! `plan_wiring_conflict` is a pure function over the rustc-free [`WiringConflict`] model, so it
//! is driven directly over hand-built conflicts — no compiler, no diagnostic wrapper. The driver
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
        "[CGP-E004] duplicate wiring for component `GreeterComponent` on `Person`",
    );
}

#[test]
fn duplicate_path_key() {
    let conflict = WiringConflict::Duplicate {
        context: "App".to_owned(),
        key: WiringKey::Path("@cgp.core.error.ErrorTypeProviderComponent.*".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E004] duplicate wiring for `@cgp.core.error.ErrorTypeProviderComponent.*` on `App`",
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
        "[CGP-E005] `App` cannot wire component `ErrorTypeProviderComponent` that is already set through `DefaultNamespace`",
    );
}

#[test]
fn overlap_path_over_namespace() {
    let conflict = WiringConflict::Overlap {
        context: "App".to_owned(),
        conflicting: WiringKey::Path("@app.GreeterComponent.*".to_owned()),
        first: WiringKey::Blanket("AppNamespace".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E005] `App` cannot wire `@app.GreeterComponent.*` that is already set through `AppNamespace`",
    );
}

#[test]
fn overlap_path_prefix_of_path() {
    let conflict = WiringConflict::Overlap {
        context: "App".to_owned(),
        conflicting: WiringKey::Path(
            "@cgp.core.error.ErrorTypeProviderComponent.String.*".to_owned(),
        ),
        first: WiringKey::Path("@cgp.core.error.ErrorTypeProviderComponent.*".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E005] `App` cannot wire `@cgp.core.error.ErrorTypeProviderComponent.String.*` that is already set through `@cgp.core.error.ErrorTypeProviderComponent.*`",
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
        "[CGP-E006] only one namespace can be used for each target type in `delegate_components!`, but `App` uses both `DefaultNamespace` and `GreeterTable`",
    );
}

#[test]
fn redirect_collides_with_direct_wiring() {
    // The fix moves to a `help`, keeping the header a single short sentence.
    let conflict = WiringConflict::Redirect {
        context: "Person".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
        path: "@GreeterComponent".to_owned(),
        provider: "GreetHello".to_owned(),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E007] component `GreeterComponent` on `Person` is redirected to `@GreeterComponent`",
    );
    assert_eq!(
        wiring_conflict_help(&conflict).as_deref(),
        Some("wire the provider `GreetHello` with the key `@GreeterComponent`"),
    );
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
        "[CGP-E008] duplicate redirect for component `FooComponent` on `App`: redirected to both `@app.foo` and `@app.bar`",
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
        "[CGP-E008] duplicate redirect for component `GreeterComponent` on `Person` (redirected to `@GreeterComponent`)",
    );
}
