//! `open` plus a `UseDelegate` table for the same component.
//!
//! The opened route looks `AreaCalculatorComponent` up with its parameter `Shape` appended, so the
//! `[CGP-E013]` help names `@AreaCalculatorComponent.<Shape>` keys in place of the direct
//! `UseDelegate<AreaTable>` entry, rather than the bare path the lookup never reads.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).

//@quality: acceptable
//@codes: CGP-E013

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
