//! A `MatchWithHandlers` list missing the handler for one variant.
//!
//! An `E0277` on `__PartialShape<IsVoid, IsPresent>: FinalizeExtract`, which names no missing
//! variant. rustc reports it twice at one caret, and the tool merges the repeat into the first
//! block.
//!
//! CGP error class: none yet.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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

use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};

fn area(shape: Shape) -> f64 {
    MatchWithHandlers::<
        Product![ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>],
    >::compute(&(), PhantomData::<()>, shape)
}

fn main() {}
