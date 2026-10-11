//! `MatchWithValueHandlers` applied to a struct.
//!
//! A `[CGP-E201]` leaf on `HasFieldHandlers<…>`; nothing says the matchers need an enum.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E201

use core::marker::PhantomData;

use cgp::prelude::*;

#[derive(CgpData)]
pub struct Scene {
    pub width: f64,
}

pub struct ComputeArea;

#[cgp_provider]
impl<Context, Code> Computer<Context, Code, f64> for ComputeArea {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, value: f64) -> f64 {
        value
    }
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent: MatchWithValueHandlers<ComputeArea>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), Scene),
    }
}

fn main() {}
