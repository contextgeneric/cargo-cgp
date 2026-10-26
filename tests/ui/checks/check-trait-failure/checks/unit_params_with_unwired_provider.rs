//! A component with a type parameter, wired with `delegate_and_check_components!` and no
//! `#[check_params(...)]`, beside a second provider of the component that the context does not
//! wire.
//!
//! `RectangleArea` provides `CanCalculateArea<Rectangle>` and `CircleArea` provides
//! `CanCalculateArea<Circle>`, and `App` wires only `RectangleArea`, so the derived check at the unit
//! parameters fails. The tool reports `[CGP-E060]`, and its help names only `Rectangle`: `App`
//! cannot be checked at `Circle`, which a provider it does not wire implements.
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

pub struct Circle {
    pub radius: f64,
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<Rectangle> {
    fn area(&self, shape: &Rectangle) -> f64 {
        shape.width * shape.height
    }
}

#[cgp_impl(new CircleArea)]
impl AreaCalculator<Circle> {
    fn area(&self, shape: &Circle) -> f64 {
        3.0 * shape.radius * shape.radius
    }
}

pub struct App;

delegate_and_check_components! {
    App {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn main() {}
