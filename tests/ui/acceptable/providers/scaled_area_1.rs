//! Usability: the failing layer of a higher-order provider is not spelled out.
//!
//! `ScaledArea<RectangleArea>` wraps an inner calculator; `Rectangle` has
//! `scale_factor` but not `height`, so the *inner* `RectangleArea` layer fails. The
//! signal is present — the "introduced here" caret sits on the inner provider's
//! `where` clause and the chain runs through both providers' `IsProviderFor` — but
//! the reader must know to read it. The cause is recoverable, so this is a usability
//! problem: the tool should name the failing layer. Contrast scaled_area_2, where
//! the outer layer fails.
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
    pub scale_factor: f64,
    pub width: f64,
    // missing height field to trigger error
    // pub height: f64,
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
