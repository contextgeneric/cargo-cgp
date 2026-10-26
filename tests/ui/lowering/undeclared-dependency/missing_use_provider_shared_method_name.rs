//! Two higher-order providers call an inner provider's `area` without `#[use_provider]`, where two
//! components' provider traits declare an `area`.
//!
//! `ScaledArea` implements `AreaCalculator`, which declares `area`, so the tool reports `[CGP-E031]`
//! naming that trait. `DescribeArea` implements `Describer`, which does not, so the tool cannot tell
//! which of the two traits is meant, and leaves rustc's `E0599`, whose suggestions name the generated
//! `__Context__`.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: usability
//@codes: CGP-E031

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(FloorAreaCalculator)]
pub trait CanCalculateFloorArea {
    fn area(&self) -> f64;
}

#[cgp_component(Describer)]
pub trait CanDescribe {
    fn describe(&self) -> String;
}

// The impl's own trait declares `area`, and so does `FloorAreaCalculator`.
#[cgp_impl(new ScaledArea<Inner>)]
impl<Inner> AreaCalculator {
    fn area(&self) -> f64 {
        Inner::area(self) * 2.0
    }
}

// The impl's own trait does not declare `area`, and two provider traits do.
#[cgp_impl(new DescribeArea<Inner>)]
impl<Inner> Describer {
    fn describe(&self) -> String {
        format!("{}", Inner::area(self))
    }
}

fn main() {}
