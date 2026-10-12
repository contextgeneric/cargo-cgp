//! A component named with a parameter and a `#[cgp_impl]` that relies on the default component name.
//!
//! The `E0107` carries a garbage suggestion (`#[cgp_impl(new SquareArea)]<Shape>`); the
//! `CGP-E032`/`CGP-E033` rewrites correctly stay off.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component {
    name: AreaCalculatorComponent<Shape>,
    provider: AreaCalculator,
}]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Square(pub f64);

#[cgp_impl(new SquareArea)]
impl AreaCalculator<Square> {
    fn area(&self, shape: &Square) -> f64 {
        shape.0 * shape.0
    }
}

fn main() {}
