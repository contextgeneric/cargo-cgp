//! A compiler crash after a buffered diagnostic keeps rustc's output order.
//!
//! The program has a CGP wiring failure (`Rectangle` lacks the `height` field `RectangleArea`
//! reads) and then a plain type error. `-Ztreat-err-as-bug=2` makes the compiler panic on the
//! second error, as an internal compiler error. The emitter holds the wiring failure in its buffer
//! until the end of the compilation, so the driver's panic hook prints the buffer, untransformed,
//! before rustc's crash report: the wiring failure comes first, as plain rustc prints it. Without
//! the hook, the crash report would come first and the buffered errors after it.
//!
//! `RUSTC_ICE=0` stops rustc writing a crash-report file into the fixture's crate, and
//! `RUST_BACKTRACE=0` keeps the backtrace out of the snapshot.
//!
//! Error class: none; this pins how the tool degrades when the compiler itself crashes.

//@env: RUSTFLAGS=-Ztreat-err-as-bug=2
//@env: RUSTC_ICE=0
//@env: RUST_BACKTRACE=0

use cgp::prelude::*;

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
    "not a number"
}

fn main() {
    let _ = after();
}
