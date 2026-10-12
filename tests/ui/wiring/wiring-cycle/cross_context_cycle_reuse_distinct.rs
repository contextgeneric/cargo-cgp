//! A wiring cycle through two contexts whose errors share no cause, so a cached subtree reused
//! where it would cut the walk shows in the output. `Inner`'s `DoN` needs the plain trait bound
//! `Outer: Linked`; `Linked` holds for `Outer` only through `CanM`, whose provider needs
//! `Inner: CanN` back. `Outer`'s `DoR` also reads a `name` field `Outer` lacks.
//!
//! The walk from `Inner: CanN` reads `Outer: Linked` as a leaf and caches the complete subtree. The
//! walk from `Outer: CanR` descends `Outer: Linked`, reaches `Inner: CanN`, and finds that cached
//! subtree. Its reachable set holds `Outer: Linked`, which is now an ancestor, so the walk resolves
//! the node afresh and the cycle guard cuts the loop: `Outer`'s only cause is its missing `name`.
//! Reusing the cached subtree would splice the looping `Outer: Linked` leaf into `Outer`'s error.
//!
//! The two errors stay apart, since an ordinary bound is grouped with the context it was reached
//! from and the missing field is `Outer`'s alone. That is what lets this fixture pin the reuse rule,
//! which `cross_context_cycle_reuse` pins no longer: there the two errors share a missing field and
//! merge into one block, whose union of causes would hide the splice.
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
pub struct Inner {
    pub name: String,
}

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
#[uses(Linked, HasName)]
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
