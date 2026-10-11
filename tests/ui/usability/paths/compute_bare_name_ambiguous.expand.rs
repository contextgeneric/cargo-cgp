#![feature(prelude_import)]
//! `compute` called by bare name where two traits provide it.
//!
//! rustc's `E0034` suggestions print `cgp::cgp_extra::cgp_handler::CanCompute` under the tool and
//! `<App as cgp::prelude::Computer>` without its generics.
//!
//! CGP error class: none; a driver path-printing defect affecting untransformed diagnostics.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
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
