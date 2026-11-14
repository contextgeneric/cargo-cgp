//! A component named with a parameter and a `#[cgp_impl]` that relies on the default component name.
//!
//! The component's key is `AreaCalculatorComponent<Shape>`, but the header `impl AreaCalculator<Square>`
//! makes the macro derive `AreaCalculatorComponent` without its parameter, so rustc reports an
//! `E0107` "missing generics" on the trait's name and suggests `AreaCalculator<Shape><Square>`,
//! which does not reach the key. The tool reshapes it into `[CGP-E070]`, naming the key with the
//! header's argument in the attribute, `#[cgp_impl(new SquareArea: AreaCalculatorComponent<Square>)]`.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E070

use cgp::prelude::*;

#[cgp_component {
    name: AreaCalculatorComponent<Shape>,
    provider: AreaCalculator,
}]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Square(pub f64);

#[cgp_impl(new SquareArea)]
impl AreaCalculator<Square> {
    fn area(&self, shape: &Square) -> f64 {
        shape.0 * shape.0
    }
}

fn main() {}
