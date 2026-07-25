//! Usability: the wiring-note and header rewrites over a component with *three* generic
//! parameters, so the parameters reach `CanUseComponent`/`IsProviderFor` grouped in a tuple.
//!
//! `CanCalculateArea<Scale, Offset, Unit>` has three type parameters, so the check is
//! `Rectangle: CanUseComponent<AreaCalculatorComponent, (u32, u64, bool)>` — the parameters
//! arrive as one tuple argument. The provider still needs a `width` field the `Rectangle`
//! lacks, so the failure surfaces through the same chain as `generic_area`, but now the
//! header must *unwrap* the tuple to name the trait as written (`CanCalculateArea<u32, u64,
//! bool>`). This fixture is the regression guard for that multi-parameter unwrapping.
//!
//! Exposes issues in cgp-knowledge-base/cargo-cgp/issues/usability.md. CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Scale, Offset, Unit> {
    fn area(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasWidth {
    fn width(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<u32, u64, bool>
where
    Self: HasWidth,
{
    fn area(&self) -> f64 {
        self.width()
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
        AreaCalculatorComponent: (u32, u64, bool),
    }
}

fn main() {}
