//! A panic in the emitter's flush falls back to the diagnostics rustc built.
//!
//! The program has a CGP wiring failure (`Rectangle` lacks the `height` field `RectangleArea`
//! reads) between two plain type errors. The fixture makes the emitter panic on purpose while it
//! renders its buffered diagnostics at the end of the compilation, so the wiring failure is shown
//! as rustc reported it, with a note naming the internal error, and the two type errors are shown
//! unchanged. Without the containment, the panic aborts the process with no diagnostic printed.
//!
//! Error class: none; this pins how the tool degrades when its own flush fails.

//@env: CARGO_CGP_TEST_PANIC=flush

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
