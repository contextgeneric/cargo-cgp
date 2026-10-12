//! A higher-order provider of a generic component calling its inner provider without importing it.
//!
//! `Inner::area(self, shape)` needs `Inner: AreaCalculator<Self, Shape>`, declared with
//! `#[use_provider(Inner: AreaCalculator<Shape>)]`. The tool reshapes rustc's `E0599` into
//! `[CGP-E031]`, and its help writes the component's parameter from the call: the `shape`
//! argument's declared `&Shape` fixes it, and the context argument is left for the attribute to
//! supply. Without the parameter the import fails with `E0107`.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E031

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

#[cgp_impl(new ScaledArea<Inner>)]
impl<Shape, Inner> AreaCalculator<Shape> {
    fn area(&self, shape: &Shape) -> f64 {
        Inner::area(self, shape) * 2.0
    }
}

fn main() {}
