//! A wiring cycle through two contexts, whose shared node is cached under one walk and must not be
//! reused under another that would cut it. `Inner`'s `DoN` needs a `name` field `Inner` lacks and
//! the plain trait bound `Outer: Linked`; `Linked` holds for `Outer` only through `CanM`, whose
//! provider needs `Inner: CanN` back.
//!
//! The walk from `Inner: CanN` reads `Outer: Linked` as a leaf, since a plain bound on another
//! type is not descended, and caches the complete subtree. The walk from `Outer: CanR` descends
//! `Outer: Linked` (a bound on its own context), reaches `Inner: CanN`, and finds that cached
//! subtree. Its reachable set holds `Outer: Linked`, which is now an ancestor, so the walk resolves
//! the node afresh and the cycle guard cuts the loop.
//!
//! Both errors bottom out on `Inner`'s missing `name` field, so they merge into one block with a
//! caret on one check entry and a label on the other. The merged tree shows both routes and
//! `Inner`'s own `Outer: Linked` leaf, so the merged output does not show whether `Outer`'s walk
//! reused the cached subtree; `cross_context_cycle_reuse_distinct` pins that rule instead.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`); the reuse rule is in
//! `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110 CGP-E117

use cgp::prelude::*;

#[cgp_component(ProviderN)]
pub trait CanN {
    fn n(&self);
}

#[cgp_component(ProviderM)]
pub trait CanM {
    fn m(&self);
}

#[cgp_component(ProviderR)]
pub trait CanR {
    fn r(&self);
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

pub trait Linked {}

impl<Context> Linked for Context where Context: CanM {}

#[derive(HasField)]
pub struct Inner {}

#[derive(HasField)]
pub struct Outer {}

#[cgp_impl(new DoN<Partner>)]
#[uses(HasName)]
impl<Partner> ProviderN
where
    Partner: Linked,
{
    fn n(&self) {
        let _ = self.name();
    }
}

#[cgp_impl(new DoM<Partner>)]
impl<Partner> ProviderM
where
    Partner: CanN,
{
    fn m(&self) {}
}

#[cgp_impl(new DoR)]
#[uses(Linked)]
impl ProviderR {
    fn r(&self) {}
}

delegate_components! {
    Inner {
        ProviderNComponent: DoN<Outer>,
    }
}

delegate_components! {
    Outer {
        ProviderMComponent: DoM<Inner>,
        ProviderRComponent: DoR,
    }
}

check_components! {
    Inner {
        ProviderNComponent,
    }
}

check_components! {
    Outer {
        ProviderRComponent,
    }
}

fn main() {}
