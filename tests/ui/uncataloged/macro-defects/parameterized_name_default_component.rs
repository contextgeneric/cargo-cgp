//! A component named with a parameter and a `#[cgp_impl]` that relies on the default component name.
//!
//! The `E0107` carried a suggestion spliced after the attribute (`#[cgp_impl(new SquareArea)]<Shape>`),
//! which the tool drops; the `CGP-E032`/`CGP-E033` rewrites correctly stay off. What remains does not
//! say that the component name defaults from the provider trait's name.
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
