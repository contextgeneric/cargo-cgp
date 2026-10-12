//! `#[uses(AreaCalculator)]` on a `#[cgp_fn]`, naming a provider trait where the consumer trait
//! belongs.
//!
//! rustc reports `E0107`, missing generics for trait `AreaCalculator`, and suggests
//! `AreaCalculator<__Context__>`, which leaks the generated name and is the wrong fix. The tool
//! reshapes it into `[CGP-E037]`, naming the consumer trait `CanCalculateArea` to declare, with
//! `#[uses(CanCalculateArea)]` in a `help`.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E037

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_fn]
#[uses(AreaCalculator)]
pub fn double_area(&self) -> f64 {
    self.area() * 2.0
}

fn main() {}
