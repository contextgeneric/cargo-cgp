//! A provider header for a component whose key takes a parameter, written without the trait's own
//! argument.
//!
//! `impl AreaCalculator` omits the `Shape` argument the provider trait takes, so rustc reports an
//! `E0107` "missing generics for trait `AreaCalculator`" on the trait's name, beside the one the
//! derived key `AreaCalculatorComponent` gets at the same name. Only the key's is `[CGP-E070]`,
//! whose attribute form keeps the key's parameter `Shape`, since the header writes no argument for
//! it; the trait's own arity mistake keeps rustc's error.
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

#[cgp_impl(new AnyArea)]
impl AreaCalculator {
    fn area(&self, _shape: &u32) -> f64 {
        1.0
    }
}

fn main() {}
