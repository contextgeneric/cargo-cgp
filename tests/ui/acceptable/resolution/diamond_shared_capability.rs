//! Diamond reuse in the resolver's walk: one shared capability reached from two independent
//! branches of a single dependency tree. `CanTop` depends on both `CanLeft` and `CanRight`, and
//! each of those depends on the same `CanShared` capability, whose provider needs the `name` field.
//! `App` wires all four components but has no `name` field, so the walk from `CanTop` descends into
//! `App: CanShared` twice — once under `CanLeft`, once under `CanRight` — the diamond the
//! per-node [resolution cache](../../../../docs/implementation/cached-dependency-resolution.md)
//! resolves once and reuses.
//!
//! Because both branches bottom out on the *same* missing field, the per-leaf de-duplication keeps
//! a single root cause, and the tree shown is the first branch that reached it (`CanLeft`). The
//! point of the fixture is that the shared `CanShared` subtree renders identically whichever branch
//! reaches it first, so a cache hit on the second branch is output-preserving — the whole-suite
//! guard the cache soundness rests on, made explicit on a minimal diamond.
//!
//! See docs/implementation/cached-dependency-resolution.md (diamond reuse).

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
