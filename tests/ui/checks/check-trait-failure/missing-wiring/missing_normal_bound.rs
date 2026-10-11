//! A checked dependency chain that ends in an ordinary trait bound the context lacks.
//!
//! `DoFooWithBar` uses `CanUseBar`, whose provider `DoBar` uses `Clone`, and `App` does not derive
//! `Clone`. `delegate_and_check_components!` checks both components, and the tool reports one
//! `[CGP-E001]` naming both consumer traits over a `[CGP-E201]` root cause, `App: Clone`, with the
//! chain through both providers.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E201

use cgp::prelude::*;

#[cgp_component(FooProvider)]
pub trait CanUseFoo {
    fn foo(&self);
}

#[cgp_component(BarProvider)]
pub trait CanUseBar {
    fn bar(&self);
}

#[cgp_impl(new DoBar)]
#[uses(Clone)]
impl BarProvider {
    fn bar(&self) {}
}

#[cgp_impl(new DoFooWithBar)]
#[uses(CanUseBar)]
impl FooProvider {
    fn foo(&self) {
        self.bar()
    }
}

delegate_components! {
    new CommonProvider {
        FooProviderComponent: DoFooWithBar,
    }
}

delegate_and_check_components! {
    new App {
        FooProviderComponent: DoFooWithBar,
        BarProviderComponent: DoBar,
    }
}

fn main() {}
