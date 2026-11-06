//! `#[uses(AreaCalculator<Rectangle>)]` on a `#[cgp_impl]` provider, naming a generic component's
//! provider trait where its consumer trait belongs.
//!
//! The provider trait `AreaCalculator<Context, Shape>` takes the context first, so the one argument
//! written fills the context's place and rustc reports `E0107`, two generic arguments expected and
//! one supplied, with a help adding `Shape`. The tool reshapes it into `[CGP-E037]`, whose help names
//! the consumer trait with the arguments written, `#[uses(CanCalculateArea<Rectangle>)]`.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: usability
//@codes: CGP-E030

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
#[uses(AreaCalculator<Rectangle>)]
impl Doubler {
    fn double(&self) -> f64 {
        self.area(&Rectangle) * 2.0
    }
}

fn main() {}
