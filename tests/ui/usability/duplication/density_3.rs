//! Usability: one root cause reported as many errors.
//!
//! Both `AreaCalculatorComponent` and `DensityCalculatorComponent` are checked, and
//! the single missing `height` field produces two full `E0277` cascades — the error
//! count reflects the depth of the wiring graph, not the number of mistakes. The
//! cause is present in both, so this is a usability problem: the tool should
//! deduplicate, coalescing every block with the same unmet bound into one headline
//! and reporting the count of affected components.
//!
//! Exposes issues in docs/issues/usability.md. CGP error class:
//! https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/verbose-cascade.md.

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

#[cgp_auto_getter]
pub trait HasMass {
    fn mass(&self) -> f64;
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
            RectangleArea,
        DensityCalculatorComponent:
            DensityFromMassField,
    }
}

// Missing height field causes RectangleArea -> DensityFromMassField to fail

check_components! {
    Rectangle {
        AreaCalculatorComponent,
        DensityCalculatorComponent,
    }
}

fn main() {}
