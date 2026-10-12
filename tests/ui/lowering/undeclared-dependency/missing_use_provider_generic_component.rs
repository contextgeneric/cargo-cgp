//! A higher-order provider of a generic component calling its inner provider without importing it.
//!
//! `Inner::area(self, shape)` needs `Inner: AreaCalculator<Self, Shape>`, declared with
//! `#[use_provider(Inner: AreaCalculator<Shape>)]`. The tool reshapes rustc's `E0599` into
//! `[CGP-E031]`, but its help names the provider trait without the component's parameter,
//! `#[use_provider(Inner: AreaCalculator)]`, which fails with `E0107` (the trait takes two generic
//! arguments and receives one).
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
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
