//! Acceptable failure: a *transitive* missing wiring. `DoFooWithBar` carries the
//! impl-side dependency `#[uses(CanUseBar)]`, so `App` can only use `FooProvider`
//! if it also wires the `BarProvider` component — but `App` wires only
//! `FooProviderComponent`, never `BarProviderComponent`. The check therefore fails
//! not because a field is missing but because a component the wired provider needs
//! is not delegated at all.
//!
//! This is the missing-wiring analog of acceptable/fields/missing_dependency.rs: the
//! typed resolver walks the same `CanUseComponent` → `IsProviderFor` chain, but the
//! terminal leaf is an unmet `DelegateComponent<BarProviderComponent>` on the context
//! rather than an unmet `HasField`. It renders as a `[CGP-E001]` header over one
//! `root cause: context \`App\` does not contain any delegate entry for \`BarProviderComponent\`` note, with the
//! dependency chain bottoming out at the `CanUseBar` capability the missing component
//! would supply.
//!
//! See cgp-knowledge-base/cgp/errors/checks/check-trait-failure.md (its "the wiring
//! is missing" face) and
//! cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md.

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

// `App` wires `FooProviderComponent` but forgets `BarProviderComponent`, which
// `DoFooWithBar` depends on — so the check fails here.
delegate_and_check_components! {
    new App {
        FooProviderComponent: DoFooWithBar,
    }
}

fn main() {}
