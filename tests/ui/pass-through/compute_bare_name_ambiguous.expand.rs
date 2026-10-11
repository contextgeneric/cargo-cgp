#![feature(prelude_import)]
//! `compute` called by bare name where two traits provide it.
//!
//! The error is plain Rust: the call is ambiguous (`E0034`). The tool must print what plain
//! `cargo check` prints, keeping the `cgp::prelude::Computer` path rustc prints in a note.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;
    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}
impl<Context, Code> IsProviderFor<ComputerComponent, Context, (Code, u64)> for Double {}
pub struct Double;
pub struct App;
use cgp::extra::handler::CanCompute;
impl DelegateComponent<ComputerComponent> for App {
    type Delegate = Double;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerComponent, __Context__, __Params__> for App
where
    Double: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
fn main() {
    let _ = App::compute(&App, PhantomData::<()>, 21);
}
