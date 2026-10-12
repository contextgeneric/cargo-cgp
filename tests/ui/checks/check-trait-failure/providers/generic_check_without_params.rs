//! A generic component checked without its parameter.
//!
//! The `check_components!` entry names no parameters, so it checks `CanCalculateArea` at the unit
//! parameters `()`, which `SquareArea`, written for `Square`, does not provide. The tool reports
//! `[CGP-E060]` with `AreaCalculatorComponent: Square` in the help, where rustc reports
//! `SquareArea: IsProviderFor<AreaCalculatorComponent, App>`, which reads like a broken provider.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E060

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
