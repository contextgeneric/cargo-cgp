#![feature(prelude_import)]
//! A pipeline stage taking `u64` after one returning `String`.
//!
//! A `[CGP-E002]` headline with rustc's "expected `u64`, found `String`" help, no tree, and no mention
//! of which stages disagree.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::extra::handler::PipeHandlers;
use cgp::prelude::*;
impl<Context, Code> Computer<Context, Code, u64> for ToText {
    type Output = String;
    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> String {
        input.to_string()
    }
}
impl<Context, Code> IsProviderFor<ComputerComponent, Context, (Code, u64)> for ToText {}
pub struct ToText;
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;
    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}
impl<Context, Code> IsProviderFor<ComputerComponent, Context, (Code, u64)> for Double {}
pub struct Double;
pub struct App;
impl DelegateComponent<ComputerComponent> for App {
    type Delegate = PipeHandlers<Product![ToText, Double]>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerComponent, __Context__, __Params__> for App
where
    PipeHandlers<
        Product![ToText, Double],
    >: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ComputerComponent, ((), u64)> for App {}
fn main() {}
