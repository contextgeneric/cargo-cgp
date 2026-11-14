//! `compute` called by bare name where two traits provide it.
//!
//! The error is plain Rust: the call is ambiguous (`E0034`), because both `CanCompute` and the
//! imported `Computer` provide `compute`. The tool must print what plain `cargo check` prints.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use core::marker::PhantomData;

use cgp::extra::handler::{Computer, ComputerComponent};
use cgp::prelude::*;

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

pub struct App;

use cgp::extra::handler::CanCompute;

delegate_components! {
    App {
        ComputerComponent: Double,
    }
}

fn main() {
    let _ = App::compute(&App, PhantomData::<()>, 21);
}
