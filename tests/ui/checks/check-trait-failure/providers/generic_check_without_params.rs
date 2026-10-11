//! A generic component checked without its parameter.
//!
//! A `[CGP-E002]` headline over rustc's "expected `Square`, found `()`" help, with no tree and no hint
//! that the check entry needs the parameter.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E002

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
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

pub struct App;

delegate_components! {
    App {
        AreaCalculatorComponent: SquareArea,
    }
}

check_components! {
    App {
        AreaCalculatorComponent,
    }
}

fn main() {}
