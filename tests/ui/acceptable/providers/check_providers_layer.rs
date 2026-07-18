//! A `#[check_providers(...)]` per-layer assertion failing on the outer layer.
//!
//! The provider-side check form asserts `IsProviderFor` on each listed provider
//! directly, so a broken layer of a higher-order stack errors on its own line. The
//! diagnostic opens on the `IsProviderFor` bound itself — there is no
//! `CanUseComponent` check impl for the typed resolver to anchor on — so this pins
//! the text-rewrite path: the `[CGP-E002]` provider-form header naming the failing
//! layer, over rustc's own notes.

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

#[cgp_auto_getter]
pub trait HasScaleFactor {
    fn scale_factor(&self) -> f64;
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

#[cgp_impl(new ScaledArea<InnerCalculator>)]
#[use_provider(InnerCalculator: AreaCalculator)]
impl<InnerCalculator> AreaCalculator
where
    Self: HasScaleFactor,
{
    fn area(&self) -> f64 {
        self.scale_factor() * InnerCalculator::area(self)
    }
}

#[derive(HasField)]
pub struct Rectangle {
    // missing scale_factor field, so only the outer `ScaledArea` layer fails
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
    #[check_trait(CheckRectangleProviders)]
    #[check_providers(
        RectangleArea,
        ScaledArea<RectangleArea>,
    )]
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
