//! Acceptable failure: a missing wiring surfaced at the *use site* rather than at a
//! check. `App` wires `FooProviderComponent: DoFooWithBar` but never wires
//! `BarProviderComponent`, which `DoFooWithBar` depends on. CGP wiring is lazy, so the
//! gap does not surface at the `delegate_components!` block; it surfaces here, when
//! `foo()` is called, as an `E0599` "method exists but its trait bounds were not
//! satisfied".
//!
//! This is the missing-wiring analog of acceptable/use-site/missing_dependency.rs. With
//! no check impl to anchor on, the resolver recovers the context `App` from the
//! diagnostic's spans and re-checks the component it *does* wire
//! (`FooProviderComponent`), walking down to the unwired `BarProviderComponent`. The
//! result is the same `[CGP-E001]` header (the code kept `E0599`) over a
//! `root cause: context \`App\` does not contain any delegate entry for \`BarProviderComponent\`` note, with
//! rustc's misleading "use associated function syntax instead" advice dropped.
//!
//! See cgp-knowledge-base/cgp/errors/hidden/unsatisfied-dependency.md and
//! cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md (the use-site path).

use cgp::prelude::*;

#[cgp_component(FooProvider)]
pub trait CanUseFoo {
    fn foo(&self);
}

#[cgp_component(BarProvider)]
pub trait CanUseBar {
    fn bar(&self);
}

#[cgp_impl(new DoFooWithBar)]
#[uses(CanUseBar)]
impl FooProvider {
    fn foo(&self) {
        self.bar()
    }
}

pub struct App;

// Accepted even though `App` never wires `BarProviderComponent`, which `DoFooWithBar`
// needs.
delegate_components! {
    App {
        FooProviderComponent: DoFooWithBar,
    }
}

fn main() {
    // The missing wiring is reported here, at the call site.
    App.foo();
}
