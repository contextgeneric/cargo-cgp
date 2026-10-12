//! `delegate_and_check_components!` wiring a component with a type parameter, without
//! `#[check_params(...)]`.
//!
//! The derived check tests `CanCalculateArea` at the unit parameters `()`, which `RectangleArea`,
//! written for `Rectangle`, does not provide. The tool reports `[CGP-E060]` with
//! `#[check_params(Rectangle)]` in the help, where rustc reports the provider as not implementing
//! the provider trait and only its help "expected `Rectangle`, found `()`" hints at the cause.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E060

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<Rectangle> {
    fn area(&self, shape: &Rectangle) -> f64 {
        shape.width * shape.height
    }
}

pub struct App;

// error[E0277]: the derived check tests `AreaCalculatorComponent` at unit parameters
delegate_and_check_components! {
    App {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn main() {}
