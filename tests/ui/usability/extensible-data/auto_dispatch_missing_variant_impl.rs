//! A `#[cgp_auto_dispatch]` trait implemented for only one of an enum's two
//! payload types. The generated blanket impl requires every variant's payload to
//! implement the trait, so calling the method on the enum fails with `E0599` at
//! the call, whose notes list the matcher's unsatisfied `Computer` bound rather
//! than naming the payload, `Square`, that lacks the impl.
//!
//! The tool passes the error through as rustc wrote it, so a reader is told only
//! that `HasArea` exists on `Circle` and that a bound on the generated matcher
//! `MatchWithValueHandlersRef<ComputeArea>` failed. Naming the variant whose
//! payload lacks the impl is the presentation work outstanding.
//!
//! Error class: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md, with the headline under
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/error_codes/e0599.md.

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
