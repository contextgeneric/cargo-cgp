//! A higher-order provider calls its inner provider without `#[use_provider]`, passing an argument
//! whose type the tool cannot read without type checking (`&Some(make_square()).unwrap()`).
//!
//! The tool reports `[CGP-E031]`, and since the call does not give the component's parameter away,
//! the help writes the provider trait with the trait's own parameter name, `AreaCalculator<Shape>`,
//! and says to put the type the call uses in its place.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E031

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Square;

fn make_square() -> Square {
    Square
}

#[cgp_impl(new SquareArea<Inner>)]
impl<Inner> AreaCalculator<Square> {
    fn area(&self, _shape: &Square) -> f64 {
        Inner::area(self, &Some(make_square()).unwrap())
    }
}

fn main() {}
