//! Usability: a field whose name matches but whose type does not, read through a getter trait.
//! `Rectangle` derives `HasField`, so `HasField<Symbol!("height")>` *is* implemented — but with
//! `Value = i32`, while the `height()` getter needs `Value = f64`. The trait bound holds; only the
//! associated-type projection `<Rectangle as HasField<Symbol!("height")>>::Value == f64` fails,
//! which rustc reports as an `E0271` type mismatch.
//!
//! The driver resolves this the way it resolves a missing field: it walks the wiring to the failing
//! `HasField` projection, queries the struct (by `DefId`) for the field's actual type `i32`, and
//! rewrites the main message into the `[CGP-E003]` field-type-mismatch form — `expected a `height`
//! field of type `f64` on `Rectangle`, but found `i32`` — over the dependency chain, which here runs
//! through the `HasRectangleFields` getter trait. The `E0271` Rust code is kept.
//!
//! The shorter-chain counterpart, where the provider reads the field directly with an `#[implicit]`
//! argument (no getter trait), is [`field_type_mismatch_1`](field_type_mismatch_1.rs); the
//! module-path disambiguation is [`field_type_mismatch_modules`](field_type_mismatch_modules.rs).

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

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: i32,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
