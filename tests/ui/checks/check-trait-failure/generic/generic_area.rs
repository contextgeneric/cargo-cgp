//! A component that carries a generic parameter, resolved through the typed walk, so
//! `CanUseComponent`/`IsProviderFor` gain an extra type argument.
//!
//! `CanCalculateArea<Scalar>` is generic, so its provider trait is
//! `AreaCalculator<Context, Scalar>` and the check is
//! `Rectangle: CanUseComponent<AreaCalculatorComponent, f64>`. The provider depends on a `width`
//! field the `Rectangle` lacks, so the failure surfaces through the same `IsProviderFor` /
//! `CanUseComponent` chain as `base_area_1`, with the extra `f64` parameter in the wiring traits.
//! The fixture guards that the typed resolution names the traits when the component carries a
//! generic parameter.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Scalar> {
    fn area(&self) -> Scalar;
}

#[cgp_auto_getter]
pub trait HasWidth {
    fn width(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<f64>
where
    Self: HasWidth,
{
    fn area(&self) -> f64 {
        self.width() * self.width()
    }
}

#[derive(HasField)]
pub struct Rectangle {
    // missing `width` field to trigger the error
    pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent: f64,
    }
}

fn main() {}
