//! Usability: a field whose name matches but whose type does not, read *directly* by a provider —
//! the shorter-chain counterpart of [`field_type_mismatch`](field_type_mismatch.rs). `RectangleArea`
//! takes `height` as an `#[implicit]` argument of type `f64`, so `HasField<Symbol!("height")>` *is*
//! implemented for `Rectangle` (it derives `HasField`) but with `Value = i32`; the trait bound
//! holds and only the associated-type projection `<Rectangle as HasField<Symbol!("height")>>::Value
//! == f64` fails, an `E0271` type mismatch.
//!
//! With no getter trait in between, the failing projection sits directly on the provider's
//! `IsProviderFor`, so the resolved chain is one node shorter than
//! [`field_type_mismatch`](field_type_mismatch.rs): `CanCalculateArea → AreaCalculator/RectangleArea
//! → HasField height`. The driver still queries the struct by `DefId` for the actual type `i32` and
//! rewrites the main message into the `[CGP-E003]` field-type-mismatch form, keeping the `E0271`
//! Rust code.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator
{
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
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
