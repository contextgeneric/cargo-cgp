//! A `#[cgp_impl]` header naming the consumer trait, called on a context that lacks a field.
//!
//! Two mistakes: `impl CanCalculateArea` should be `impl AreaCalculator`, and `Rectangle` lacks the
//! `height` field the provider reads. No check lists the component, so the field surfaces only at
//! the call `rectangle.area()`, whose `E0599` names the impl's unmet `HasField` bound among its
//! method-probe notes. The tool reports the header as `[CGP-E032]` and resolves the call through the
//! impl's requirements as written, so the call reads `[CGP-E001] missing field height`, the error
//! the program gives once the header is fixed.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E032 CGP-E101 CGP-E102 CGP-E110

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl CanCalculateArea {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent:
            RectangleArea,
    }
}

fn main() {
    let rectangle = Rectangle { width: 2.0 };
    let _ = rectangle.area();
}
