//! `delegate_and_check_components!` wiring a component with a type parameter, without
//! `#[check_params(...)]`.
//!
//! The derived check tests `CanCalculateArea` at the unit parameters `()`, which `RectangleArea`,
//! written for `Rectangle`, does not provide. The text rewrite reports a `[CGP-E002]` provider-trait
//! failure on `RectangleArea`, and only rustc's help "expected `Rectangle`, found `()`" hints that the
//! entry names no parameters. Nothing names `#[check_params(Rectangle)]` as the fix.
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
