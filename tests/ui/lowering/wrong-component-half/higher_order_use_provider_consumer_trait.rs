//! A `#[use_provider]` bound naming the consumer trait where the provider trait belongs.
//!
//! `#[use_provider(InnerCalculator: CanCalculateArea)]` should name `AreaCalculator`. The attribute
//! inserts the context as the trait's leading generic, so the consumer trait receives one argument
//! too many and rustc reports `E0107`. The tool reshapes it into `[CGP-E034]`, naming the provider
//! trait to import.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E034

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

// MISTAKE: `#[use_provider]` names the consumer trait `CanCalculateArea` where the provider trait
// `AreaCalculator` belongs. The attribute inserts the context as the leading generic, generating the
// bound `InnerCalculator: CanCalculateArea<Self>` — but the consumer trait takes no context
// parameter, so it is given one argument too many.
#[cgp_impl(new ScaledArea<InnerCalculator>)]
#[use_provider(InnerCalculator: CanCalculateArea)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        InnerCalculator::area(self) * scale_factor * scale_factor
    }
}

fn main() {}
