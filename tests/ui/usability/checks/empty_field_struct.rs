//! Edge case (expected behavior): a context derives `HasField` but has *no fields at
//! all*, while a getter names a field.
//!
//! `#[derive(HasField)]` emits one `HasField` impl per field, so on a struct with no
//! fields it emits *nothing* — byte-for-byte the same as a struct with no derive at all.
//! A fieldless derive leaves no trace in the generated program, so it is impossible to
//! tell whether the derive was even written; the two are the same program wherever
//! `HasField` is concerned. The diagnostic is correspondingly identical to the
//! missing-derive case (`base_area_2.rs`): the first field (`width`) is unimplemented,
//! there is **no** "but trait `HasField<…>` is implemented for it" landmark, and the
//! `help` points at the `struct` definition.
//!
//! So cargo-cgp reporting that `#[derive(HasField)]` is required here is fine, not a
//! misclassification: it accurately states what is observable — the context implements
//! `HasField` for no field — and a fieldless derive is exactly that. There is nothing to
//! recover, since the two situations are the same program. This fixture pins the expected
//! behavior; see docs/implementation/error-processing.md.
//!
//! Exposes issues in docs/issues/usability.md. CGP error class:
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
