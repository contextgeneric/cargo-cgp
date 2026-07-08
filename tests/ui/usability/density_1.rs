//! Usability: the dependency path from the checked component to the cause is buried.
//!
//! `DensityFromMassField` depends on `CanCalculateArea`, which `RectangleArea`
//! provides from `width`/`height`; `Rectangle` omits `height`. The check names
//! `DensityCalculatorComponent`, but the missing field belongs to the transitive
//! `AreaCalculator` dependency, and the connection is spelled only through a stack
//! of `required for …` notes over `IsProviderFor`/`CanUseComponent`. The cause is
//! present, so this is a usability problem: the tool should collapse the chain to a
//! short path (Density → Area → missing `height`) and drop the scaffolding.
//!
//! Exposes issues in docs/issues/usability.md. CGP error class:
//! ../../../../cgp/docs/errors/checks/verbose-cascade.md.

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
        DensityCalculatorComponent,
    }
}

fn main() {}
