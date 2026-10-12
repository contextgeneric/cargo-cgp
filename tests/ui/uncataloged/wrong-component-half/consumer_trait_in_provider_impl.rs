//! A `#[cgp_impl]` header naming the consumer trait where the provider trait belongs.
//!
//! `impl CanCalculateArea` should be `impl AreaCalculator`. The macro inserts the context as the
//! provider trait's leading generic, so the consumer trait receives one argument too many and
//! rustc reports an `E0107` arity error among unrelated codes. The tool reshapes it into
//! `[CGP-E032]`, naming both halves and the header to write. It is the one-line minimal pair of
//! `base_area_1_1`, which writes the header correctly.
//!
//! `Rectangle` also lacks the `height` field the provider reads, a second mistake the compiler
//! reports on the check through the provider's requirements. The tool drops that error as a
//! consequence of the header, so the missing field surfaces only once the header is fixed.
//!
//! CGP error class: the wrong half of a component, not yet in the catalog
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md#a-gap-the-used-but-undeclared-and-wrong-half-classes).

//@quality: usability
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
    // missing height field to trigger error
    // pub height: f64,
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
