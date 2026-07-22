//! Two independent consumers coalesced on one cause via parallel (non-subsuming) chains.
//!
//! `CanCalculateArea` and `CanReportHeight` are unrelated — neither depends on the other
//! — yet both read the `height` field the `Rectangle` context is missing, so both fail
//! with the same root cause. cargo-cgp coalesces them into a single `[CGP-E001]` block
//! naming both consumers, with a caret per check entry and the shared cause shown once.
//!
//! Unlike `duplication/density_3.rs`, where `CanCalculateDensity`'s chain *subsumes*
//! `CanCalculateArea`'s (density depends on area) and the merged block must lead with the
//! deeper chain, here neither chain contains the other: they are equal-depth parallel
//! branches to the same missing field, each through its own getter and provider. With no
//! subsuming chain to prefer, the representative is the first check entry
//! (`AreaCalculatorComponent`) — a deterministic choice that does not flip on entry order.
//! This pins the parallel counterpart of the subsuming coalescing the cascade fixtures pin.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/verbose-cascade.md.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(HeightReporter)]
pub trait CanReportHeight {
    fn report_height(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasRectangleFields {
    fn width(&self) -> f64;

    fn height(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasHeight {
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

#[cgp_impl(new ReportHeight)]
impl HeightReporter
where
    Self: HasHeight,
{
    fn report_height(&self) -> f64 {
        self.height()
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    // missing height field to trigger error
    // pub height: f64,
}

// `RectangleArea` reads `height` through `HasRectangleFields`, `ReportHeight` through the
// separate `HasHeight` getter — two independent paths to the one missing field.

delegate_components! {
    Rectangle {
        AreaCalculatorComponent:
            RectangleArea,
        HeightReporterComponent:
            ReportHeight,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
        HeightReporterComponent,
    }
}

fn main() {}
