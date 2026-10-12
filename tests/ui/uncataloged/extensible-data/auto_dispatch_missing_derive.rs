//! A `#[cgp_auto_dispatch]` call on an enum without `#[derive(CgpData)]`.
//!
//! rustc's `E0599` names `MatchWithValueHandlersRef<ComputeArea>` and a source path in
//! `cgp-dispatch`. The tool resolves the call to its two unmet requirements on the enum,
//! `Shape: HasFields` and `Shape: HasExtractor`, the traits the derive supplies, but nothing names
//! the derive itself, and a step of the tree names the matcher's private
//! `dispatch::providers::matchers::to_field_handlers` path.
//!
//! CGP error class: none yet.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E003 CGP-E102 CGP-E104 CGP-E117

use cgp::prelude::*;

pub struct Circle {
    pub radius: f64,
}

pub enum Shape {
    Circle(Circle),
}

#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}

impl HasArea for Circle {
    fn area(&self) -> f64 {
        self.radius
    }
}

fn main() {
    let _ = Shape::Circle(Circle { radius: 1.0 }).area();
}
