//! Usability: a missing `#[derive(HasField)]` reads like a single missing field.
//!
//! `Rectangle` has `width` and `height` but no `#[derive(HasField)]`, so it has no
//! `HasField` impls at all. The check reports only the first field (`width`) as
//! unimplemented and, unlike the other fixtures, carries no "but trait `HasField<…>`
//! is implemented for it" landmark — because `Rectangle` implements the trait for no
//! field. The root cause is recoverable (a field that exists yet is "not
//! implemented", plus the absent landmark, points at the missing derive), so this is
//! a usability problem: the tool should say "add #[derive(HasField)]" rather than
//! send the user to add one field at a time.
//!
//! Exposes issues in docs/issues/usability.md. CGP error class:
//! ../../../../cgp/docs/errors/checks/check-trait-failure.md (derive-missing variant).

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
