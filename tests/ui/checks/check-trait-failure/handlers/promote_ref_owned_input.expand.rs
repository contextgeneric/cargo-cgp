#![feature(prelude_import)]
//! `PromoteRef<Double>` for a `Double` that takes its input by value.
//!
//! `PromoteRef` calls its inner provider with a reference, so the walk reaches
//! `Double: Computer<App, (), &u64>`, and the compiler's `E0271` on its output becomes
//! ``[CGP-E127] `Double` implements `Computer` only with `Input = u64`, not `Input = &u64` ``.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
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
use cgp::extra::handler::PromoteRef;
impl DelegateComponent<ComputerRefComponent> for App {
    type Delegate = PromoteRef<Double>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for App
where
    PromoteRef<Double>: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ComputerRefComponent, ((), u64)> for App {}
fn main() {}
