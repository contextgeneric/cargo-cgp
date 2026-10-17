//! `UseDelegate` wired for a component that lacks `#[derive_delegate]`.
//!
//! `[CGP-E115]` says `UseDelegate` is not a provider of the component, and its `help` names the
//! `open` statement, which dispatches on the component's parameters without `#[derive_delegate]`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E115

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
        AreaCalculatorComponent: UseDelegate<new AreaTable {
            Square: SquareArea,
        }>,
    }
}

check_components! {
    App {
        AreaCalculatorComponent: Square,
    }
}

fn main() {}
