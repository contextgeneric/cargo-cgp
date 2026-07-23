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
