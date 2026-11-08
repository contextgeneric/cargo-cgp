#![feature(prelude_import)]
//! `PromoteComputer<Double>` wired as a handler for a hand-written `Computer` provider.
//!
//! `[CGP-E115]` says `Double` does not implement `TryComputer`, and its `help` names the adapter
//! that makes it one, `Promote<Double>`, with the `use` it needs, which the solver confirms.
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
///Provider `Double` for [`Computer`].
pub struct Double;
pub struct App;
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for App {
    type Delegate = PromoteComputer<Double>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerComponent, __Context__, __Params__> for App
where
    PromoteComputer<Double>: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<HandlerComponent, ((), u64)> for App {}
fn main() {}
