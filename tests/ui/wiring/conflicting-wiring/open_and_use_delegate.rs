//! `open` plus a `UseDelegate` table for the same component.
//!
//! `[CGP-E007]` helps "wire the provider `UseDelegate<AreaTable>` with the key
//! `@AreaCalculatorComponent`", which would wire a dispatcher under the opened route; the fix is one
//! dispatch mechanism per component.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E007

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
#[derive_delegate(UseDelegate<Shape>)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle;

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<Rectangle> {
    fn area(&self, _shape: &Rectangle) -> f64 {
        1.0
    }
}

pub struct App;

delegate_components! {
    App {
        open AreaCalculatorComponent;

        @AreaCalculatorComponent.Rectangle: RectangleArea,
    }
}

delegate_components! {
    App {
        AreaCalculatorComponent: UseDelegate<new AreaTable {
            Rectangle: RectangleArea,
        }>,
    }
}

fn main() {}
