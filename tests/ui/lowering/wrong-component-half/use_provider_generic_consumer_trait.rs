//! A `#[use_provider]` bound naming a generic component's consumer trait where its provider trait
//! belongs.
//!
//! `#[use_provider(Inner: CanCalculateArea<Shape>)]` should name `AreaCalculator<Shape>`. The
//! attribute inserts the context ahead of `Shape`, so the consumer trait receives one argument too
//! many and rustc reports `E0107`. The bound sits on the inner provider with the context as its
//! first argument, the shape the attribute writes, so the tool reshapes it into `[CGP-E034]` as it
//! does for a component without parameters.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E034

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

// MISTAKE: `#[use_provider]` names the consumer trait `CanCalculateArea` where the provider trait
// `AreaCalculator` belongs, generating `Inner: CanCalculateArea<Self, Shape>`.
#[cgp_impl(new DoubledArea<Inner>)]
#[use_provider(Inner: CanCalculateArea<Shape>)]
impl<Inner, Shape> AreaCalculator<Shape> {
    fn area(&self, shape: &Shape) -> f64 {
        Inner::area(self, shape) * 2.0
    }
}

fn main() {}
