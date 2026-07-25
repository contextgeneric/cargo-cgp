//! Acceptable failure: two independent missing wirings surface as two separate root
//! causes. `DoFooWithBarBaz` depends on both `CanUseBar` and `CanUseBaz`
//! (`#[uses(CanUseBar, CanUseBaz)]`), and `App` wires neither `BarProviderComponent`
//! nor `BazProviderComponent`, so both are missing.
//!
//! This is the missing-wiring analog of acceptable/fields/parallel_branches.rs: the
//! resolver follows *every* unmet dependency, not just the first the next-generation
//! solver stops at, so a single `[CGP-E001]` header carries two `root cause: missing
//! wiring …` notes — one per unwired component — each with its own dependency chain.
//! A regression that followed only the first unmet bound would report one and hide the
//! other.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md
//! (parallel branches).

use cgp::prelude::*;

#[cgp_component(FooProvider)]
pub trait CanUseFoo {
    fn foo(&self);
}

#[cgp_component(BarProvider)]
pub trait CanUseBar {
    fn bar(&self);
}

#[cgp_component(BazProvider)]
pub trait CanUseBaz {
    fn baz(&self);
}

#[cgp_impl(new DoFooWithBarBaz)]
#[uses(CanUseBar, CanUseBaz)]
impl FooProvider {
    fn foo(&self) {
        self.bar();
        self.baz();
    }
}

// `App` wires neither `BarProviderComponent` nor `BazProviderComponent`, both of which
// `DoFooWithBarBaz` needs — so the check reports both as missing wirings.
delegate_and_check_components! {
    new App {
        FooProviderComponent: DoFooWithBarBaz,
    }
}

fn main() {}
