//! One root cause reported once, listing every affected component.
//!
//! Both `AreaCalculatorComponent` and `DensityCalculatorComponent` are checked, and a
//! single missing `height` field breaks both. Left to rustc that is two full `E0277`
//! blocks — an error count reflecting the depth of the wiring graph, not the number of
//! mistakes. cargo-cgp coalesces the two: they share one root cause, so they collapse
//! into a single `[CGP-E001]` headline naming both consumer traits, with one caret per
//! failing check entry and the shared root cause shown once. This is the *different
//! consumers, one cause* coalescing — distinct from the same-consumer de-duplication
//! `duplication/cross_site_dedup.rs` pins.
//!
//! CGP error class:
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
