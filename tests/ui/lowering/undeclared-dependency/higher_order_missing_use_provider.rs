//! A higher-order provider calling its inner provider without importing it.
//!
//! `ScaledArea<InnerCalculator>` calls `InnerCalculator::area(self)` without
//! `#[use_provider(InnerCalculator: AreaCalculator)]`, so `InnerCalculator` carries no
//! `AreaCalculator<Self>` bound and rustc reports `E0599` on a type parameter. The tool reshapes it
//! into `[CGP-E031]`, naming the inner provider and the attribute that imports it.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E031

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

// MISTAKE: the higher-order provider `ScaledArea` calls its inner provider as
// `InnerCalculator::area(self)`, but never imports it with
// `#[use_provider(InnerCalculator: AreaCalculator)]`. Without that, `InnerCalculator` carries no
// `AreaCalculator<Self>` bound, so the associated-function call cannot resolve.
#[cgp_impl(new ScaledArea<InnerCalculator>)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        InnerCalculator::area(self) * scale_factor * scale_factor
    }
}

fn main() {}
