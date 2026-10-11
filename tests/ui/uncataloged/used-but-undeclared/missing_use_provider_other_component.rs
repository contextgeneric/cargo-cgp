//! A higher-order provider calling an inner provider of another component without importing it.
//!
//! `ScaledArea<InnerShape>` implements `AreaCalculator` but calls `InnerShape::perimeter(self)`, a
//! method of the `PerimeterCalculator` provider trait, without
//! `#[use_provider(InnerShape: PerimeterCalculator)]`. The enclosing impl's own trait does not
//! declare `perimeter`, so the provider trait to import is found among every provider trait that
//! declares the method. rustc reports `E0599` on the unbounded type parameter.
//!
//! CGP error class: a used-but-undeclared dependency, not yet in the catalog
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md#a-gap-the-used-but-undeclared-and-wrong-half-classes).

//@quality: acceptable
//@codes: CGP-E016

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(PerimeterCalculator)]
pub trait CanCalculatePerimeter {
    fn perimeter(&self) -> f64;
}

// MISTAKE: `InnerShape::perimeter(self)` needs `#[use_provider(InnerShape: PerimeterCalculator)]`.
#[cgp_impl(new ScaledArea<InnerShape>)]
impl<InnerShape> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        InnerShape::perimeter(self) * scale_factor
    }
}

fn main() {}
