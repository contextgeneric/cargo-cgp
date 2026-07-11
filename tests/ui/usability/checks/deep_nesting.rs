//! Usability: a deeply nested stack of higher-order providers, whose dependency tree should
//! descend through every layer down to the missing field at the bottom.
//!
//! `ScaledArea` wraps an inner `AreaCalculator`, and the wiring nests it three deep around
//! `RectangleArea`. `Rectangle` has `scale_factor` and `width` but not `height`, so the
//! innermost `RectangleArea` layer fails — through all three `ScaledArea` layers. The dependency
//! note should show the full nesting as one spine.
//!
//! CGP error class: ../../../../../cgp/docs/errors/checks/higher-order-provider-layer.md.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
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

#[derive(HasField)]
pub struct Rectangle {
    pub scale_factor: f64,
    pub width: f64,
    // missing height field, so the innermost RectangleArea layer fails
    // pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent:
            ScaledArea<ScaledArea<ScaledArea<RectangleArea>>>,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
