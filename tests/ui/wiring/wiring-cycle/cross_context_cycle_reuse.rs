//! A wiring cycle through two contexts, whose shared node is cached under one walk and must not be
//! reused under another that would cut it. `Inner`'s `DoN` needs a `name` field `Inner` lacks and
//! the plain trait bound `Outer: Linked`; `Linked` holds for `Outer` only through `CanM`, whose
//! provider needs `Inner: CanN` back.
//!
//! The walk from `Inner: CanN` reads `Outer: Linked` as a leaf, since a plain bound on another
//! type is not descended, and caches the complete subtree. The walk from `Outer: CanR` descends
//! `Outer: Linked` (a bound on its own context), reaches `Inner: CanN`, and finds that cached
//! subtree. Its reachable set holds `Outer: Linked`, which is now an ancestor, so the walk resolves
//! the node afresh and the cycle guard cuts the loop. Reusing the cached subtree would splice the
//! looping `Outer: Linked` leaf into `Outer`'s tree.
//!
//! The `Outer` error is `usability`: its headline rewords the deep obligation rustc reports,
//! `Inner: IsProviderFor<ProviderNComponent, Inner>`, as a provider trait not implemented for
//! provider `Inner`, under a tree rooted at `CanR`.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`); the reuse rule is in
//! `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.

//@quality: usability
//@codes: CGP-E001 CGP-E002 CGP-E101 CGP-E102 CGP-E105 CGP-E106 CGP-E201

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
