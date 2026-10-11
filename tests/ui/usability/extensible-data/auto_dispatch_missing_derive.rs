//! A `#[cgp_auto_dispatch]` call on an enum without `#[derive(CgpData)]`.
//!
//! A raw `E0599` naming `MatchWithValueHandlersRef<ComputeArea>` and a source path in
//! `cgp-dispatch`; nothing says the derive is missing.
//!
//! CGP error class: none yet.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
