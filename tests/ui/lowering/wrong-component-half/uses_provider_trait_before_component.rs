//! `#[uses(AreaCalculator)]` on a `#[cgp_fn]` written before the component it names.
//!
//! rustc reports `E0107` while it lowers the generated blanket impl's `where` clause, and the search
//! for the provider trait's consumer scans every trait. The search skips the impl's own trait, whose
//! blanket impl is the one being lowered, and reports `[CGP-E037]`; without the skip, this program
//! keeps rustc's `E0107`.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E037

use cgp::prelude::*;

#[cgp_fn]
#[uses(AreaCalculator)]
pub fn doubled_area(&self) -> f64 {
    self.area() * 2.0
}

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

fn main() {}
