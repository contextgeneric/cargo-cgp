//! Usability: the wiring-note and header rewrites over a component that carries a generic
//! parameter, so `CanUseComponent`/`IsProviderFor` gain an extra type argument.
//!
//! `CanCalculateArea<Scalar>` is generic, so its provider trait is `AreaCalculator<Context,
//! Scalar>` and the check is `Rectangle: CanUseComponent<AreaCalculatorComponent, f64>`. The
//! provider still depends on a `width` field the `Rectangle` lacks, so the failure surfaces
//! through the same `IsProviderFor` / `CanUseComponent` chain as `base_area_1` — but now with
//! the extra `f64` parameter in the wiring traits. This fixture is the regression guard that
//! the driver's trait-renaming still names the traits when generic parameters are present.
//!
//! Exposes issues in docs/issues/usability.md. CGP error class:
//! ../../../../../cgp/docs/errors/checks/check-trait-failure.md.

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
