//! `delegate_and_check_components!` called by its path, wiring a component with a type parameter
//! without `#[check_params(...)]`.
//!
//! The table is written `cgp::prelude::delegate_and_check_components! { … }`, so the compiler
//! records the macro under that whole path. The tool reports `[CGP-E060]`, but its help names the
//! `check_components!` form, `AreaCalculatorComponent: Rectangle`, which in this table would wire
//! `Rectangle` as the provider; the bare call (`generic_component_unit_params`) gets
//! `#[check_params(Rectangle)]`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: usability
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

cgp::prelude::delegate_and_check_components! {
    App {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn main() {}
