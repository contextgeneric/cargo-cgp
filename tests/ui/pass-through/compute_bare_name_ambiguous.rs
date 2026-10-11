//! `compute` called by bare name where two traits provide it.
//!
//! The error is plain Rust: the call is ambiguous (`E0034`). The tool must print what plain
//! `cargo check` prints, keeping the `cgp::prelude::Computer` path rustc prints in a note.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
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
