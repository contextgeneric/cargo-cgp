use cgp::prelude::*;

// A component with a generic parameter: the consumer trait `CanCalculateArea<Shape>` pairs with the
// provider trait `AreaCalculator<Context, Shape>`.
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: Shape) -> f64;
}

pub struct Rectangle;

// MISTAKE: the `#[cgp_impl]` header names the consumer trait `CanCalculateArea` where the provider
// trait `AreaCalculator` belongs. The macro inserts the context as the leading generic, so the
// consumer trait is given one argument too many — a generic component reproduces the same class of
// error as a parameterless one.
#[cgp_impl(new RectangleArea)]
impl CanCalculateArea<Rectangle> {
    fn area(&self, _shape: Rectangle) -> f64 {
        1.0
    }
}

fn main() {}
