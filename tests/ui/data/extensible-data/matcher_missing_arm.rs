//! A `MatchWithHandlers` list missing the handler for one variant, its `compute` called through
//! the provider's own path.
//!
//! The provider-call anchor seeds `MatchWithHandlers<…>: Computer<(), (), Shape>` from the call,
//! and the walk reads the extractor's presence markers behind the failing `FinalizeExtract`:
//! ``[CGP-E002] variant `Rectangle` of `Shape` is not extracted``. rustc reports the failure twice
//! at one caret, and the repeat resolves to the same cause and is dropped.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E002 CGP-E102 CGP-E121

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
