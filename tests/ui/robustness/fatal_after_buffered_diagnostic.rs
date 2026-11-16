//! A fatal error after a transformed diagnostic still prints both.
//!
//! The program has a CGP wiring failure (`Rectangle` lacks the `height` field `RectangleArea`
//! reads), which the tool rewrites and holds in its buffer until the compilation ends, followed by
//! two constants defined through each other. Evaluating them is a query cycle (`E0391`), which
//! rustc reports as a fatal error and stops on. The buffered diagnostic is printed while the fatal
//! error unwinds, before the cycle error, in the order plain rustc prints them.
//!
//! Error class: none; this pins that buffering survives a fatal error.
//!
//! Under threads, rustc's cycle error starts from whichever constant a thread reached first, so its
//! text differs between runs, and a run under `--threads` other than one skips the tool's snapshot.

//@varies-under-threads: rustc's `E0391` cycle starts from whichever constant a thread reached first

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
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
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

const A: usize = B;
const B: usize = A;

fn main() {
    let _ = A;
}
