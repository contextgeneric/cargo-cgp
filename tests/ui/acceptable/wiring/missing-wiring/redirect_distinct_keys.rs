//! Acceptable: two dependencies dispatched along the same `open` route for distinct, unwired value
//! types — the shape that pins two redirect hops staying distinct nodes across branches.
//!
//! `AssembleParts` (wired directly to `AssemblerComponent`, *not* through a redirect) needs to build
//! two values, a `Left` and a `Right`, through the `open`-dispatched `ValueBuilder`. Neither type is
//! wired, so checking `CanAssemble` fails with two root causes: the missing
//! `@ValueBuilderComponent.Left` and `@ValueBuilderComponent.Right` wirings. Each is reached through a
//! `redirect lookup to @ValueBuilderComponent` hop — the same route, rendered identically — but for a
//! different dispatch key.
//!
//! Because the top component is wired directly, each branch's redirect is the *first* redirect on its
//! path, so the two are compared for identity against each other. They render the same label yet are
//! different lookups; the dependency graph keys a redirect node's identity on the dispatched key as
//! well as the route, so the two stay distinct: the tree branches to each value's own redirect and
//! its own missing-wiring leaf, rather than collapsing both leaves under one shared redirect with the
//! other branch reduced to a `(*)` back-reference.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/dependency-graph-rendering.md (node identity is
//! cross-path, keyed on the dispatched value).

use cgp::prelude::*;

// The per-value build capability, dispatched per value type with `open`.
#[cgp_component(ValueBuilder)]
pub trait CanBuildValue<Value> {
    fn build_value(&self) -> Value;
}

#[cgp_impl(new BuildU64)]
impl ValueBuilder<u64> {
    fn build_value(&self) -> u64 {
        0
    }
}

// Two distinct value types, both left unwired — the two distinct missing keys.
pub struct Left;
pub struct Right;

// A top capability wired *directly* to its provider, so its dependencies' redirects are each the
// first redirect on their path.
#[cgp_component(Assembler)]
pub trait CanAssemble {
    fn assemble(&self);
}

#[cgp_impl(new AssembleParts)]
#[uses(CanBuildValue<Left>, CanBuildValue<Right>)]
impl Assembler {
    fn assemble(&self) {
        let _: Left = self.build_value();
        let _: Right = self.build_value();
    }
}

pub struct App;

delegate_components! {
    App {
        open ValueBuilderComponent;

        @ValueBuilderComponent.u64: BuildU64,
        // `Left` and `Right` are deliberately left unwired — two distinct missing keys reached
        // through the same `@ValueBuilderComponent` redirect route.

        AssemblerComponent: AssembleParts,
    }
}

check_components! {
    App {
        AssemblerComponent,
    }
}

fn main() {}
