//! `#[uses(AreaCalculator)]`, naming a provider trait where the consumer trait belongs.
//!
//! The `E0107` "missing generics" keeps rustc's help `AreaCalculator<__Context__>`, which leaks the
//! generated name and is the wrong fix.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_fn]
#[uses(AreaCalculator)]
pub fn double_area(&self) -> f64 {
    self.area() * 2.0
}

fn main() {}
