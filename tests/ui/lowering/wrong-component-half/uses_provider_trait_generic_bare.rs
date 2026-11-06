//! `#[uses(AreaCalculator)]` on a `#[cgp_impl]` provider, naming a generic component's provider
//! trait with no arguments where its consumer trait belongs.
//!
//! rustc reports `E0107`, missing generics for trait `AreaCalculator`, as for a component without
//! parameters, and the tool reshapes it into `[CGP-E037]`. The program wrote no argument for the
//! component's `Shape`, so the help names the consumer trait bare, `#[uses(CanCalculateArea)]`,
//! which still needs the argument the program means.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E037

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

#[cgp_component(Doubler)]
pub trait CanDouble {
    fn double(&self) -> f64;
}

pub struct Rectangle;

#[cgp_impl(new DoubleArea)]
#[uses(AreaCalculator)]
impl Doubler {
    fn double(&self) -> f64 {
        self.area(&Rectangle) * 2.0
    }
}

fn main() {}
