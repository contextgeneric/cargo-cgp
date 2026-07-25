//! Acceptable failure: the simplest missing wiring — a `check_components!` asserts a
//! component the context does not wire at all. There is no transitive dependency here;
//! `App` has an empty wiring table, so `App: DelegateComponent<FooProviderComponent>`
//! is unmet directly under the checked `CanUseComponent` obligation.
//!
//! The dependency chain is therefore a single node — the `CanUseFoo` consumer the
//! missing wiring would provide — with the same `[CGP-E001]` header and a
//! `root cause: context \`App\` does not contain any delegate entry for \`FooProviderComponent\`` note. It pins
//! that the resolver reports a bare unwired component, not only one reached through a
//! provider's impl-side dependency (that transitive case is basic_missing_wiring.rs).
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md.

use cgp::prelude::*;

#[cgp_component(FooProvider)]
pub trait CanUseFoo {
    fn foo(&self);
}

#[cgp_impl(new DoFoo)]
impl FooProvider {
    fn foo(&self) {}
}

pub struct App;

// `App` never wires `FooProviderComponent`, yet the check asserts it can use it.
check_components! {
    App {
        FooProviderComponent,
    }
}

fn main() {}
