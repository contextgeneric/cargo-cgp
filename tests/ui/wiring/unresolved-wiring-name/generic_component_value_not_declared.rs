//! A wiring value that names no provider, for a component with a parameter, and a call of the
//! component.
//!
//! `AreaCalculatorComponent: RectangleAreaa` names no type, so rustc reports an `E0425` at the
//! value, and the call then overflows (`E0275`) on `App: CanCalculateArea<Rectangle>`, a
//! requirement the entry leaves unprovable. The tool reports one `[CGP-E043]` at the value and
//! drops the overflow as its consequence, matching the requirement with the component's parameters
//! after the consumer trait's name.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E043

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle;

pub struct App;

delegate_components! {
    App {
        AreaCalculatorComponent: RectangleAreaa,
    }
}

fn main() {
    let _ = App.area(&Rectangle);
}
