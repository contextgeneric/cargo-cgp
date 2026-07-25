//! The negative counterpart of [`abstract_type_mismatch`](abstract_type_mismatch.rs): the same
//! projection failure on a trait that is **not** a CGP abstract-type component. `HasUnit` is a
//! plain Rust trait with an associated type, implemented directly on the context — no
//! `#[cgp_type]`, so no provider trait, no component marker, and no `UseType` blanket.
//! `RectangleArea` pins its `Unit` to `f64` through an ordinary `where` clause (the form
//! [`#[use_type]`](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/reference/attributes/use_type.md)
//! deliberately does not cover, since the trait is not an abstract-type component), while
//! `Rectangle` supplies `u32`.
//!
//! The failure is reshaped into the same `[CGP-E017]` class — the mechanism is the projection, not
//! the trait — but the wording turns on the trait's fingerprint, and that is what this fixture pins.
//! Because `HasUnit`'s provider trait carries no `UseType` impl, it reads `associated type` rather
//! than `abstract type` and carries **no** `help`: there is no wiring entry to change, so suggesting
//! `UseType<f64>` here would name a fix that does not exist. The concrete type is fixed by the
//! `impl HasUnit for Rectangle` block, and that is where a reader must go.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

/// A plain associated-type trait — deliberately *not* `#[cgp_type]`, so nothing about it is wired.
pub trait HasUnit {
    type Unit;
}

pub struct Rectangle;

impl HasUnit for Rectangle {
    // The mistake: the context supplies `u32` where `RectangleArea` needs `f64`.
    type Unit = u32;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator
where
    Self: HasUnit<Unit = f64>,
{
    fn area(&self) -> f64 {
        1.0
    }
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
