//! `#[uses(AreaCalculator)]` on a `#[cgp_impl]` provider, naming a provider trait where the
//! consumer trait belongs.
//!
//! The `E0107` "missing generics" keeps rustc's help `AreaCalculator<__Context__>`, which leaks the
//! generated name and is the wrong fix: the provider's body calls `self.area()`, a method of the
//! consumer trait `CanCalculateArea`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(Doubler)]
pub trait CanDouble {
    fn double(&self) -> f64;
}

#[cgp_impl(new DoubleArea)]
#[uses(AreaCalculator)]
impl Doubler {
    fn double(&self) -> f64 {
        self.area() * 2.0
    }
}

fn main() {}
