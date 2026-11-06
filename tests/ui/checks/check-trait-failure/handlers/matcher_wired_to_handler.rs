//! A value matcher, `MatchWithValueHandlers`, wired as the context's `Handler`.
//!
//! `[CGP-E115]` is accurate but repeats a three-line provider type four times; the promotion fix is
//! not hinted.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E115

use core::marker::PhantomData;

use cgp::prelude::*;

pub struct Circle {
    pub radius: f64,
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

pub struct ComputeArea;

#[cgp_provider]
impl<Context, Code> Computer<Context, Code, Circle> for ComputeArea {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, shape: Circle) -> f64 {
        shape.radius
    }
}

#[cgp_provider]
impl<Context, Code> Computer<Context, Code, Rectangle> for ComputeArea {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, shape: Rectangle) -> f64 {
        shape.width * shape.height
    }
}

use cgp::core::error::ErrorTypeProviderComponent;

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        HandlerComponent: MatchWithValueHandlers<ComputeArea>,
    }
}

check_components! {
    App {
        HandlerComponent: ((), Shape),
    }
}

fn main() {}
