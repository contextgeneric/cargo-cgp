//! A provider header for a component whose `name:` renames its key.
//!
//! The component's key is `AreaComponent`, but the header `impl AreaCalculator` makes the macro name
//! it `AreaCalculatorComponent`, which nothing declares, so rustc reports "cannot find type
//! `AreaCalculatorComponent`" with no suggestion. The tool reshapes it into `[CGP-E070]`, naming
//! the real key and the attribute form that names it.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E070

use cgp::prelude::*;

#[cgp_component {
    name: AreaComponent,
    provider: AreaCalculator,
}]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new UnitArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        1.0
    }
}

fn main() {}
