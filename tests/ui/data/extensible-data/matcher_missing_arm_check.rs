//! A matcher wired on a context whose handler list skips one variant, asserted by a check.
//!
//! `MatchWithHandlers` extracts each listed variant in turn and finalizes the remainder, which
//! compiles only once every variant is extracted. The list handles `Circle` alone, so the remainder
//! still holds `Rectangle` and the check fails on its `FinalizeExtract` bound; the tool reads the
//! markers: ``[CGP-E001] variant `Rectangle` of `Shape` is not extracted``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E121

use core::marker::PhantomData;

use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};
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
        shape.radius * shape.radius
    }
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent: MatchWithHandlers<
            Product![ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>],
        >,
    }
}

check_components! {
    App {
        ComputerComponent: ((), Shape),
    }
}

fn main() {}
