//! A matcher wired on a context whose handler list skips one variant, asserted by a check.
//!
//! `MatchWithHandlers` extracts each listed variant in turn and finalizes the remainder, which
//! compiles only once every variant is extracted. The list handles `Circle` alone, so the remainder
//! still holds `Rectangle` and the check fails on `__PartialShape<IsVoid, IsPresent>: FinalizeExtract`;
//! the unhandled `Rectangle` is never named.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-variants.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E101 CGP-E102 CGP-E117

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
