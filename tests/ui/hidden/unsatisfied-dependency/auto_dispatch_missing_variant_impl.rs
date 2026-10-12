//! A `#[cgp_auto_dispatch]` trait implemented for only one of an enum's two payload types.
//!
//! The generated blanket impl requires every variant's payload to implement the trait, so calling
//! the method on the enum fails with `E0599` at the call, whose notes list the matcher's
//! unsatisfied `Computer` bound rather than naming the payload, `Square`, that lacks the impl.
//!
//! The call-site anchor reads the enum off the variant constructor and walks the matcher down to
//! the per-variant computer, whose requirement `Square: HasArea` is the cause: the walk stops there
//! rather than following the enum-matching blanket impl, which `Square` cannot use, so the block
//! leads with ``[CGP-E003] the trait bound `Square: HasArea` is not satisfied``.
//!
//! Error class: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md, with the headline under
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/error_codes/e0599.md.

//@quality: acceptable
//@codes: CGP-E003 CGP-E102 CGP-E104 CGP-E117

use cgp::prelude::*;

pub struct Circle;

pub struct Square;

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Square(Square),
}

#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}

impl HasArea for Circle {
    fn area(&self) -> f64 {
        3.0
    }
}

fn main() {
    let _ = Shape::Circle(Circle).area();
}
