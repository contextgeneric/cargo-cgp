//! Diamond reuse in the resolver's walk: one shared capability reached from two independent
//! branches of a single dependency tree. `CanTop` depends on both `CanLeft` and `CanRight`, and
//! each of those depends on the same `CanShared` capability, whose provider needs the `name` field.
//! `App` wires all four components but has no `name` field, so the walk from `CanTop` descends into
//! `App: CanShared` twice — once under `CanLeft`, once under `CanRight` — the diamond the per-node
//! [resolution
//! cache](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/cached-dependency-resolution.md)
//! resolves once and reuses.
//!
//! Because both branches bottom out on the *same* missing field, they are one root cause with two
//! paths, which the [dependency
//! graph](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/dependency-graph-rendering.md)
//! renders as a diamond: `CanTop` branches to `CanLeft` and `CanRight`, the shared `CanShared`
//! subtree is drawn in full under the first (`CanLeft`) and referenced with `(*)` under the second
//! (`CanRight`), and the missing `name` field is shown once. The point of the fixture is that both
//! branches appear — neither is dropped — while the shared subtree is not duplicated. It also pins
//! the cache: `CanShared` renders identically whichever branch reaches it first, so a cache hit on
//! the second branch is output-preserving.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/dependency-graph-rendering.md (diamond) and
//! cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md (diamond reuse).

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(SharedProvider)]
pub trait CanShared {
    fn shared(&self);
}

#[cgp_component(LeftProvider)]
pub trait CanLeft {
    fn left(&self);
}

#[cgp_component(RightProvider)]
pub trait CanRight {
    fn right(&self);
}

#[cgp_component(TopProvider)]
pub trait CanTop {
    fn top(&self);
}

#[cgp_impl(new ProvideShared)]
#[uses(HasName)]
impl SharedProvider {
    fn shared(&self) {
        let _ = self.name();
    }
}

#[cgp_impl(new ProvideLeft)]
#[uses(CanShared)]
impl LeftProvider {
    fn left(&self) {
        self.shared();
    }
}

#[cgp_impl(new ProvideRight)]
#[uses(CanShared)]
impl RightProvider {
    fn right(&self) {
        self.shared();
    }
}

#[cgp_impl(new ProvideTop)]
#[uses(CanLeft, CanRight)]
impl TopProvider {
    fn top(&self) {
        self.left();
        self.right();
    }
}

#[derive(HasField)]
pub struct App {
    pub age: u8,
}

delegate_components! {
    App {
        SharedProviderComponent: ProvideShared,
        LeftProviderComponent: ProvideLeft,
        RightProviderComponent: ProvideRight,
        TopProviderComponent: ProvideTop,
    }
}

check_components! {
    App {
        TopProviderComponent,
    }
}

fn main() {}
