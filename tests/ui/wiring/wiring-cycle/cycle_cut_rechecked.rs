//! A two-component wiring cycle checked at both components, so the second check re-walks a node
//! the first walk cut. `DoA` needs `CanB` and a `width` field the context lacks; `DoB` needs `CanA`
//! back. The walk from `CanA` reaches `CanB`, whose subtree loops back to `CanA` and is cut there,
//! so it is incomplete and must not be cached: the walk from `CanB` then resolves it afresh and
//! finds the missing field through `CanA`. Caching the cut subtree would leave that second walk an
//! empty result and its error unrewritten.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`); the cache rule is in
//! `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

use cgp::prelude::*;

#[cgp_component(ProviderA)]
pub trait CanA {
    fn a(&self);
}

#[cgp_component(ProviderB)]
pub trait CanB {
    fn b(&self);
}

#[cgp_auto_getter]
pub trait HasWidth {
    fn width(&self) -> f64;
}

#[cgp_impl(new DoA)]
#[uses(CanB, HasWidth)]
impl ProviderA {
    fn a(&self) {
        let _ = self.width();
        self.b();
    }
}

#[cgp_impl(new DoB)]
#[uses(CanA)]
impl ProviderB {
    fn b(&self) {
        self.a();
    }
}

#[derive(HasField)]
pub struct App {}

delegate_components! {
    App {
        ProviderAComponent: DoA,
        ProviderBComponent: DoB,
    }
}

check_components! {
    App {
        ProviderAComponent,
        ProviderBComponent,
    }
}

fn main() {}
