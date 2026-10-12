//! A `#[cgp_impl]` header naming the consumer trait, beside a field of the wrong type.
//!
//! Two mistakes: `impl CanCalculateArea` should be `impl AreaCalculator`, and `Rectangle`'s `height`
//! is an `i32` where the provider reads an `f64`. The compiler reports the field type as an `E0271`
//! on the check, through the provider's requirements. The tool reports the header as `[CGP-E032]`
//! and reads the requirements off the impl as written, so the field type is `[CGP-E004]`, the block
//! the program gives once the header is fixed.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E004 CGP-E032 CGP-E101 CGP-E102 CGP-E112

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
    pub height: i32,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent:
            RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
