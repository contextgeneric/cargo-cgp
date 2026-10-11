//! A panic in the emitter's transform falls back to the diagnostic rustc built.
//!
//! The program has a CGP wiring failure (`Rectangle` lacks the `height` field `RectangleArea`
//! reads) between two plain type errors. The fixture makes the emitter panic on purpose while it
//! transforms each error, so every error is shown as rustc reported it, each with a note naming
//! the internal error and where to report it, and the compilation finishes. Without the
//! containment, the panic stops the compilation with no diagnostic printed at all.
//!
//! Error class: none; this pins how the tool degrades when its own transform fails.

//@env: CARGO_CGP_TEST_PANIC=transform

use cgp::prelude::*;

fn before() -> u32 {
    "not a number"
}

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasRectangleFields {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
#[uses(HasRectangleFields)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        self.width() * self.height()
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn after() -> u32 {
    "also not"
}

fn main() {
    let _ = (before(), after());
}
