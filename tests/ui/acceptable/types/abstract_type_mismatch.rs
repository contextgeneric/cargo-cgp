//! An **abstract type** the context binds to one concrete type while a provider it uses pins the
//! same abstract type to another. `HasScalarType` is an
//! [abstract-type component](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/abstract-types.md):
//! generic code names `Scalar` without committing to a concrete type, and the context chooses one
//! by wiring `ScalarTypeProviderComponent` to `UseType<T>`. Here `Rectangle` wires it to
//! `UseType<u32>`, but `RectangleArea` pins it with the `#[use_type(HasScalarType.{Scalar = f64})]`
//! equality form, so the provider needs `Scalar = f64`.
//!
//! This is the abstract-type sibling of the `HasField` mismatch in
//! [`field_type_mismatch`](../field-types/field_type_mismatch.rs), and it fails the same way:
//! `Rectangle: HasScalarType` *is* implemented, so the trait bound holds and only the
//! associated-type projection `<Rectangle as HasScalarType>::Scalar == f64` fails — an `E0271`.
//!
//! The driver resolves it the same way too: the walk reaches the provider impl whose every
//! trait-clause dependency holds, finds the unmet projection it carries, normalizes
//! `<Rectangle as HasScalarType>::Scalar` to read the type the context actually supplies (`u32`),
//! and rewrites the main message into the `[CGP-E017]` abstract-type form over the dependency
//! chain. Because the trait is a `#[cgp_type]` component — recognized structurally, by its provider
//! carrying the `UseType` blanket — a `help` names the wiring entry to change. Where rustc's raw
//! output aims `expected this to be `f64`` at the `#[cgp_type]` attribute and never states the type
//! the context supplies at all, the reshaped message names both sides and the fix. The `E0271` Rust
//! code is kept.

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

#[cgp_component(AreaCalculator)]
#[use_type(HasScalarType.Scalar)]
pub trait CanCalculateArea {
    fn area(&self) -> Scalar;
}

// The provider pins the context's abstract `Scalar` to the concrete `f64`, so its body can do
// floating-point arithmetic directly.
#[cgp_impl(new RectangleArea)]
#[use_type(HasScalarType.{Scalar = f64})]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        // The mistake: the context binds `Scalar` to `u32`, but `RectangleArea` needs `f64`.
        ScalarTypeProviderComponent: UseType<u32>,
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
