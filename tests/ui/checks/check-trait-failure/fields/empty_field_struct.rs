//! Edge case (expected behavior): a context derives `HasField` but has *no fields at
//! all*, while a getter names a field.
//!
//! `#[derive(HasField)]` emits one `HasField` impl per field, so on a struct with no
//! fields it emits *nothing*: byte-for-byte the same as a struct with no derive at all.
//! A fieldless derive leaves no trace in the generated program, so it is impossible to
//! tell whether the derive was even written; the two are the same program wherever
//! `HasField` is concerned. The tool therefore reports both fields the provider reads as
//! missing, without a `help`: it cannot tell whether the fix is a field or the derive. This
//! fixture pins that output.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md
//! (derive-missing variant).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

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

// Derive present, but the struct declares no fields at all.
#[derive(HasField)]
pub struct Rectangle {}

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
