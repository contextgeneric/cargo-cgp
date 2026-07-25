//! Usability: the outer layer of a higher-order provider fails, and the diagnostic
//! does not say so outright.
//!
//! The mirror of scaled_area_1: `Rectangle` has `width`/`height` but not
//! `scale_factor`, so the *outer* `ScaledArea` fails on its own dependency and the
//! inner `RectangleArea` never runs. The output looks structurally similar to the
//! inner-failure case, but the "introduced here" caret sits on `ScaledArea`'s clause
//! and the chain is shorter (the inner provider is never named). The cause is
//! recoverable, so this is a usability problem: name the layer at fault.
//!
//! Exposes issues in cgp-knowledge-base/cargo-cgp/issues/usability.md. CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/higher-order-provider-layer.md.

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
    // missing scale_factor field to trigger error
    // pub scale_factor: f64,
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent:
            ScaledArea<RectangleArea>,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
