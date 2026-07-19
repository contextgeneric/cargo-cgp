//! A missing `#[derive(HasField)]` reported as one root cause with one fix.
//!
//! `Rectangle` has `width` and `height` but no `#[derive(HasField)]`, so it has no
//! `HasField` impls at all and the provider's getter fails for both fields. Because
//! the derive emits an impl per field, the two underived fields are one mistake with
//! one fix, and the resolver coalesces them into a single root cause listing both
//! ("accessor trait `HasField` is not implemented for the fields `height` and
//! `width` of `Rectangle`") over one merged tree whose branches still end at the
//! per-field leaves, with the derive `help` naming the fix. Pins that coalescing; the
//! boundary cases — a lone underived field, genuinely absent fields — are pinned by
//! `missing_has_field_derive`, `underived_and_missing_field`, and `parallel_branches`.
//!
//! CGP error class:
//! ../../../../../cgp/docs/errors/checks/check-trait-failure.md (derive-missing variant).

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

// Missing derive(HasField) to trigger error
// #[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent:
            RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
