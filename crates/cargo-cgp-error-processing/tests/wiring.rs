//! Tests for the duplicate-key wiring-conflict wording.
//!
//! `plan_wiring_conflict` is a pure function over the rustc-free [`WiringConflict`] model, so it
//! is driven directly over hand-built conflicts — no compiler, no diagnostic wrapper. The driver
//! fills the same model in from the live `TyCtxt`.

use cargo_cgp_error_processing::{WiringConflict, WiringKey, plan_wiring_conflict};

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
        key: WiringKey::Path("Path!(@cgp.core.error.ErrorTypeProviderComponent.*)".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E004] duplicate wiring for `Path!(@cgp.core.error.ErrorTypeProviderComponent.*)` on `App`",
    );
}

#[test]
fn overlap_two_blanket_forwardings() {
    // A namespace join plus a bare-key `for` loop: two blanket forwardings over every key.
    let conflict = WiringConflict::Overlap {
        context: "App".to_owned(),
        conflicting: WiringKey::Blanket("GreeterTable".to_owned()),
        first: WiringKey::Blanket("DefaultNamespace".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E004] `App` cannot wire a key through `GreeterTable` that is already set through `DefaultNamespace`",
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
        "[CGP-E004] `App` cannot wire component `ErrorTypeProviderComponent` that is already set through `DefaultNamespace`",
    );
}

#[test]
fn overlap_path_over_namespace() {
    let conflict = WiringConflict::Overlap {
        context: "App".to_owned(),
        conflicting: WiringKey::Path("Path!(@app.GreeterComponent.*)".to_owned()),
        first: WiringKey::Blanket("AppNamespace".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E004] `App` cannot wire `Path!(@app.GreeterComponent.*)` that is already set through `AppNamespace`",
    );
}

#[test]
fn overlap_path_prefix_of_path() {
    let conflict = WiringConflict::Overlap {
        context: "App".to_owned(),
        conflicting: WiringKey::Path(
            "Path!(@cgp.core.error.ErrorTypeProviderComponent.String.*)".to_owned(),
        ),
        first: WiringKey::Path("Path!(@cgp.core.error.ErrorTypeProviderComponent.*)".to_owned()),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E004] `App` cannot wire `Path!(@cgp.core.error.ErrorTypeProviderComponent.String.*)` that is already set through `Path!(@cgp.core.error.ErrorTypeProviderComponent.*)`",
    );
}

#[test]
fn redirect_collides_with_direct_wiring() {
    let conflict = WiringConflict::Redirect {
        context: "Person".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
        path: "Path!(@GreeterComponent)".to_owned(),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E004] component `GreeterComponent` on `Person` is redirected to `Path!(@GreeterComponent)`; set the redirected key instead of wiring it directly",
    );
}

#[test]
fn duplicate_redirect() {
    let conflict = WiringConflict::DuplicateRedirect {
        context: "Person".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
        path: "Path!(@GreeterComponent)".to_owned(),
    };
    assert_eq!(
        plan_wiring_conflict(&conflict),
        "[CGP-E004] duplicate redirect for component `GreeterComponent` on `Person` (redirected to `Path!(@GreeterComponent)`)",
    );
}
