//! `UseDelegate` wired for a component that lacks `#[derive_delegate]`.
//!
//! `[CGP-E111]` is accurate but gives no hint about the missing attribute, or that `open` needs
//! none.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
