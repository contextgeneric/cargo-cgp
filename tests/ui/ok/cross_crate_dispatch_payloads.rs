//@aux-build: cgp-test-dispatch-traits
//@aux-build: cgp-test-dispatch-shapes
//! Dispatch traits, payloads, and enums spread over three crates.
//!
//! `cgp-test-dispatch-traits` declares the `#[cgp_auto_dispatch]` traits,
//! `cgp-test-dispatch-shapes` implements them for its payloads `Circle` and
//! `Square` and derives an enum `Shape` over them, and this crate both calls the
//! dispatched methods on that foreign enum and derives its own enum over the same
//! foreign payloads, which gains the traits through the blanket impl declared two
//! crates up. The program compiles clean, so the snapshot of cargo-cgp's output is
//! empty.

use cgp::prelude::*;
use cgp_test_dispatch_shapes::{Circle, Shape, Square};
use cgp_test_dispatch_traits::{CanDescribe, HasShape};

#[derive(CgpData)]
pub enum Tile {
    Square(Square),
    Circle(Circle),
}

fn main() {
    let mut shape = Shape::Circle(Circle { radius: 1.0 });
    shape.scale(2.0);
    assert_eq!(shape.area(), 12.0);
    let _ = shape.describe();
    assert_eq!(shape.into_name(), "circle");

    let tile = Tile::Square(Square { side: 3.0 });
    assert_eq!(tile.area(), 9.0);
    assert_eq!(tile.label("a"), "a square");
    assert_eq!(tile.into_name(), "square");
}
