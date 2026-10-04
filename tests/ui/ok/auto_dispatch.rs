//! `#[cgp_auto_dispatch]` across the method shapes its expansion distinguishes:
//! a `&self` method, a `&mut self` method taking an argument, a by-value `self`
//! method, a method borrowing through an elided lifetime, and an async method
//! stacked with `#[async_trait]`. The program compiles clean, so the snapshot of
//! cargo-cgp's output is empty; the `.expand.rs` pins the enum-level blanket
//! impls and the per-variant computers the macro generates, down to the provider
//! impls their nested `#[cgp_computer]` expands to.

use cgp::prelude::*;

pub struct Circle {
    pub radius: f64,
}

pub struct Square {
    pub side: f64,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Square(Square),
}

#[cgp_auto_dispatch]
pub trait HasShape {
    fn area(&self) -> f64;

    fn scale(&mut self, factor: f64);

    fn into_name(self) -> &'static str;

    fn label(&self, prefix: &str) -> String;
}

#[cgp_auto_dispatch]
#[async_trait]
pub trait CanDescribe {
    async fn describe(&self) -> String;
}

impl HasShape for Circle {
    fn area(&self) -> f64 {
        3.0 * self.radius * self.radius
    }

    fn scale(&mut self, factor: f64) {
        self.radius *= factor;
    }

    fn into_name(self) -> &'static str {
        "circle"
    }

    fn label(&self, prefix: &str) -> String {
        format!("{prefix} circle")
    }
}

impl HasShape for Square {
    fn area(&self) -> f64 {
        self.side * self.side
    }

    fn scale(&mut self, factor: f64) {
        self.side *= factor;
    }

    fn into_name(self) -> &'static str {
        "square"
    }

    fn label(&self, prefix: &str) -> String {
        format!("{prefix} square")
    }
}

impl CanDescribe for Circle {
    async fn describe(&self) -> String {
        "a circle".to_owned()
    }
}

impl CanDescribe for Square {
    async fn describe(&self) -> String {
        "a square".to_owned()
    }
}

fn main() {
    let mut shape = Shape::Square(Square { side: 2.0 });
    shape.scale(2.0);
    assert_eq!(shape.area(), 16.0);
    assert_eq!(shape.label("big"), "big square");
    let _ = shape.describe();
    assert_eq!(shape.into_name(), "square");
}
