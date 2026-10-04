//@aux-build: cgp-test-dispatch-traits
//! Dispatch traits declared in one crate, implemented and dispatched in another.
//!
//! `cgp-test-dispatch-traits` declares the `#[cgp_auto_dispatch]` traits and
//! nothing implementing them. This crate defines the payloads, implements the
//! foreign traits for them alongside the foreign enum-level blanket impl, derives
//! the enum, and calls every dispatched method on it. It also runs the foreign
//! per-variant provider `ComputeArea` through a matcher by name, and calls the
//! foreign `#[cgp_computer]` and `#[cgp_producer]` providers. The program compiles
//! clean, so the snapshot of cargo-cgp's output is empty.

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{Computer, Producer, TryComputer};
use cgp::prelude::*;
use cgp_test_dispatch_traits::{
    CanDescribe, CheckedHalve, ComputeArea, Double, HasShape, UnitScale,
};

pub struct Triangle {
    pub base: f64,
    pub height: f64,
}

pub struct Hexagon {
    pub side: f64,
}

#[derive(CgpData)]
pub enum Polygon {
    Triangle(Triangle),
    Hexagon(Hexagon),
}

impl HasShape for Triangle {
    fn area(&self) -> f64 {
        self.base * self.height / 2.0
    }

    fn scale(&mut self, factor: f64) {
        self.base *= factor;
        self.height *= factor;
    }

    fn into_name(self) -> &'static str {
        "triangle"
    }

    fn label(&self, prefix: &str) -> String {
        format!("{prefix} triangle")
    }
}

impl HasShape for Hexagon {
    fn area(&self) -> f64 {
        2.5 * self.side * self.side
    }

    fn scale(&mut self, factor: f64) {
        self.side *= factor;
    }

    fn into_name(self) -> &'static str {
        "hexagon"
    }

    fn label(&self, prefix: &str) -> String {
        format!("{prefix} hexagon")
    }
}

impl CanDescribe for Triangle {
    async fn describe(&self) -> String {
        "a triangle".to_owned()
    }
}

impl CanDescribe for Hexagon {
    async fn describe(&self) -> String {
        "a hexagon".to_owned()
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
    }
}

fn main() {
    let mut polygon = Polygon::Triangle(Triangle {
        base: 2.0,
        height: 3.0,
    });

    polygon.scale(2.0);
    assert_eq!(polygon.area(), 12.0);
    assert_eq!(polygon.label("big"), "big triangle");
    let _ = polygon.describe();

    assert_eq!(
        <MatchWithValueHandlersRef<ComputeArea> as Computer<(), (), &Polygon>>::compute(
            &(),
            PhantomData,
            &polygon,
        ),
        12.0,
    );

    assert_eq!(polygon.into_name(), "triangle");

    let code = PhantomData::<()>;
    assert_eq!(Double::compute(&App, code, 2.0), 4.0);
    assert_eq!(CheckedHalve::try_compute(&App, code, 3), Err("3 is odd".to_owned()));
    assert_eq!(UnitScale::produce(&App, code), 1.0);
}
