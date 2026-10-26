//! An undeclared type in the path a component with a parameter is redirected to, and a check of the
//! component at a parameter.
//!
//! `AreaCalculatorComponent => @app.Missing` names a type nothing declares, so rustc reports an
//! `E0425` at each use of the path, and the check then overflows (`E0275`) on
//! `App: CanUseComponent<AreaCalculatorComponent, Rectangle>`, a requirement the redirect leaves
//! unprovable. The tool reports one `[CGP-E044]` at the redirect but keeps the overflow, though it
//! drops the same overflow for a component without parameters (`path_segment_not_declared`).
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: usability
//@codes: CGP-E044

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl<Shape> AreaCalculator<Shape> {
    fn area(&self, _shape: &Shape) -> f64 {
        0.0
    }
}

pub struct Rectangle;

pub struct App;

delegate_components! {
    App {
        AreaCalculatorComponent => @app.Missing,
        @app.Missing: RectangleArea,
    }
}

check_components! {
    App {
        AreaCalculatorComponent: Rectangle,
    }
}

fn main() {}
