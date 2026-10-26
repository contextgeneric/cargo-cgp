//! A type nothing declares, nested in a wiring value's generic arguments rather than in an
//! `@`-path.
//!
//! `AreaCalculatorComponent: UseDelegate<Missing>` names an undeclared `Missing` inside the value.
//! It is neither the whole value nor a segment of a path, so the tool leaves the compiler's
//! `E0425`s as rustc wrote them, with its suggestion to add a type parameter, which no wiring entry
//! can take.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
#[derive_delegate(UseDelegate<Shape>)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct App;

delegate_components! {
    App {
        AreaCalculatorComponent: UseDelegate<Missing>,
    }
}

fn main() {}
