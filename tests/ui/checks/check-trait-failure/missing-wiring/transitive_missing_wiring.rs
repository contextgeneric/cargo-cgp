//! A component wired to an aggregate provider that lacks one of the entries it is wired for.
//!
//! `App` wires both `FooProviderComponent` and `BarProviderComponent` to `CommonProvider`, which
//! only has an entry for `FooProviderComponent`. Checking `FooProviderComponent` reaches
//! `DoFooWithBar`, which needs `CanUseBar`, which `CommonProvider` cannot provide. The tool reports
//! `[CGP-E002]` on `CommonProvider` over a `[CGP-E110]` root cause naming the missing
//! `BarProviderComponent` entry in the aggregate rather than in `App`.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.

//@quality: acceptable
//@codes: CGP-E002 CGP-E101 CGP-E102 CGP-E110

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

delegate_components! {
    new App {
        [
            FooProviderComponent,
            BarProviderComponent,
        ]: CommonProvider,
    }
}

check_components! {
    App {
        FooProviderComponent,
    }
}

fn main() {}
