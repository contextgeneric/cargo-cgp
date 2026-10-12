//! A `#[cgp_impl]` header naming the consumer trait, where the header is the only mistake.
//!
//! `impl CanCalculateArea` should be `impl AreaCalculator`, as in `consumer_trait_in_provider_impl`,
//! but here `Rectangle` has both fields the provider reads. The call `rectangle.area()` fails only
//! because `RectangleArea` does not implement the provider trait, a consequence of the header, so the
//! tool reports the header alone as `[CGP-E032]` and drops the call's `E0599`. The check passes: the
//! provider's requirements all hold.
//!
//! CGP error class: the wrong half of a component, not yet in the catalog
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md#a-gap-the-used-but-undeclared-and-wrong-half-classes).

//@quality: acceptable
//@codes: CGP-E032

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
    pub height: f64,
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

fn main() {
    let rectangle = Rectangle {
        width: 2.0,
        height: 3.0,
    };
    let _ = rectangle.area();
}
