//! `#[uses(AreaCalculator)]` on a `#[cgp_impl]` provider, naming a provider trait where the
//! consumer trait belongs.
//!
//! rustc reports `E0107`, missing generics for trait `AreaCalculator`, and suggests
//! `AreaCalculator<__Context__>`; the provider's body calls `self.area()`, a method of the consumer
//! trait `CanCalculateArea`. The tool reshapes it into `[CGP-E037]` with
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

#[cgp_component(Doubler)]
pub trait CanDouble {
    fn double(&self) -> f64;
}

#[cgp_impl(new DoubleArea)]
#[uses(AreaCalculator)]
impl Doubler {
    fn double(&self) -> f64 {
        self.area() * 2.0
    }
}

fn main() {}
