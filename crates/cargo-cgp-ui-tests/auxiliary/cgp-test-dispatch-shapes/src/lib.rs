//! Middle crate for the cross-crate fixtures of `#[cgp_auto_dispatch]`.
//!
//! It implements the dispatch traits of `cgp-test-dispatch-traits` for its own
//! payload types, which must coexist with that crate's enum-level blanket impl,
//! and defines an extensible enum over them that gains the traits through that
//! blanket impl. A downstream fixture can then dispatch over this enum, or define
//! its own enum over these payloads.
//!
//! A fixture pulls it in with `//@aux-build: cgp-test-dispatch-shapes` (alongside
//! `//@aux-build: cgp-test-dispatch-traits`).

use cgp::prelude::*;
use cgp_test_dispatch_traits::{CanDescribe, HasShape};

pub struct Circle {
    pub radius: f64,
}

pub struct Square {
    pub side: f64,
}

/// An extensible enum over this crate's payloads.
#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Square(Square),
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
