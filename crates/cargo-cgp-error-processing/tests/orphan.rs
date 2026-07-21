//! Tests for the orphan-rule namespace-registration wording.
//!
//! `plan_orphan_conflict` and `orphan_conflict_help` are pure functions over the rustc-free
//! [`OrphanConflict`] model, so they are driven directly over hand-built conflicts — no compiler,
//! no diagnostic wrapper. The driver fills the same model in from the live `TyCtxt`. The header is
//! one `[CGP-E011]` shape for both triggers (the violation is identical); only the `help`'s fix
//! differs by whether the registration was a `#[default_impl]`/`#[prefix]` or a `cgp_namespace!`
//! re-open. A path key renders in bare `@…` notation (no `Path!(…)` wrapper).

use cargo_cgp_error_processing::{
    OrphanConflict, OrphanTrigger, WiringKey, orphan_conflict_help, plan_orphan_conflict,
};

#[test]
fn register_component_key() {
    let conflict = OrphanConflict {
        namespace: "AppNamespace".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
        trigger: OrphanTrigger::Register,
    };
    assert_eq!(
        plan_orphan_conflict(&conflict),
        "[CGP-E011] cannot register the foreign component `GreeterComponent` into the foreign namespace `AppNamespace`",
    );
    assert_eq!(
        orphan_conflict_help(&conflict),
        "own one end of the wiring: key it on a component defined in this crate, or register it from the crate that defines `AppNamespace`",
    );
}

#[test]
fn register_path_key() {
    let conflict = OrphanConflict {
        namespace: "AppNamespace".to_owned(),
        key: WiringKey::Path("@app.AnnouncerComponent".to_owned()),
        trigger: OrphanTrigger::Register,
    };
    assert_eq!(
        plan_orphan_conflict(&conflict),
        "[CGP-E011] cannot register the foreign path `@app.AnnouncerComponent` into the foreign namespace `AppNamespace`",
    );
}

#[test]
fn reopen_gives_inheritance_fix() {
    // A `cgp_namespace!` re-open has the same header but a different fix: inherit the namespace into
    // a new local one rather than owning a key.
    let conflict = OrphanConflict {
        namespace: "AppNamespace".to_owned(),
        key: WiringKey::Component("GreeterComponent".to_owned()),
        trigger: OrphanTrigger::Reopen,
    };
    assert_eq!(
        plan_orphan_conflict(&conflict),
        "[CGP-E011] cannot register the foreign component `GreeterComponent` into the foreign namespace `AppNamespace`",
    );
    assert_eq!(
        orphan_conflict_help(&conflict),
        "to extend a foreign namespace, define a new local namespace that inherits it: `cgp_namespace! { new MyNamespace: AppNamespace { … } }`",
    );
}
