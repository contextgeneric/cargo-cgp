//! An inner provider written with the consumer trait in its `#[cgp_impl]` header, under an outer
//! provider that needs a field the context lacks.
//!
//! `RectangleArea`'s header names `CanCalculateArea`, so it is not a provider of `AreaCalculator`;
//! `ScaledArea<RectangleArea>` also needs a `scale` field `Rectangle` lacks. The header is
//! `[CGP-E032]`, and the check keeps its own `[CGP-E001]` block with both causes, because one of
//! them, the missing `scale`, is not the header's consequence: a check is dropped beside the header
//! only when every cause is the not-a-provider leaf the header explains.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E032 CGP-E001 CGP-E101 CGP-E102 CGP-E110 CGP-E115

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl CanCalculateArea {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[cgp_impl(new ScaledArea<Inner>)]
#[use_provider(Inner: AreaCalculator)]
impl<Inner> AreaCalculator {
    fn area(&self, #[implicit] scale: f64) -> f64 {
        Inner::area(self) * scale
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: ScaledArea<RectangleArea>,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
