//! Usability: the same transitive-path burden as density_1, one layer deeper.
//!
//! Here `AreaCalculatorComponent` is `ScaledArea<RectangleArea>`, so the missing
//! `height` sits several hops below the checked `DensityCalculatorComponent`
//! (Density → Area → ScaledArea → RectangleArea → `height`). The extra higher-order
//! layer lengthens the `required for …` chain without adding a new cause, showing
//! that chain length tracks graph depth, not mistakes. The cause is recoverable, so
//! this is a usability problem: the summarized path must stay short even as the
//! graph deepens.
//!
//! Exposes issues in cgp-knowledge-base/cargo-cgp/issues/usability.md. CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/verbose-cascade.md.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(DensityCalculator)]
pub trait CanCalculateDensity {
    fn density(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasRectangleFields {
    fn width(&self) -> f64;

    fn height(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator
where
    Self: HasRectangleFields,
{
    fn area(&self) -> f64 {
        self.width() * self.height()
    }
}

#[cgp_auto_getter]
pub trait HasScaleFactor {
    fn scale_factor(&self) -> f64;
}

#[cgp_impl(new ScaledArea<InnerCalculator>)]
impl<InnerCalculator> AreaCalculator
where
    Self: HasScaleFactor,
    InnerCalculator: AreaCalculator<Self>,
{
    fn area(&self) -> f64 {
        self.scale_factor() * InnerCalculator::area(self)
    }
}

#[cgp_auto_getter]
pub trait HasMass {
    fn mass(&self) -> f64;
}

#[cgp_impl(new DensityFromMassField)]
impl DensityCalculator
where
    Self: CanCalculateArea + HasMass,
{
    fn density(&self) -> f64 {
        self.mass() / self.area()
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub mass: f64,
    pub width: f64,
    // missing height field to trigger error
    // pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent:
            ScaledArea<RectangleArea>,
        DensityCalculatorComponent:
            DensityFromMassField,
    }
}

// Missing height field causes RectangleArea -> ScaledArea<RectangleArea> -> DensityFromMassField to fail

check_components! {
    Rectangle {
        DensityCalculatorComponent,
    }
}

fn main() {}
