#![feature(prelude_import)]
//! `PipeMonadic<ErrMonadic, …>` over stages that do not return `Result`.
//!
//! The error monad binds each later stage with `BindErr`, which takes a `Result` input, so the walk
//! reaches `BindErr<IdentMonadic, Double>` given the first stage's `u64`:
//! ``[CGP-E127] `BindErr<IdentMonadic, Double>` implements `Computer` only with `Input = Result<_, _>`, not `Input = u64` ``.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
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
use cgp::extra::monad::monadic::err::ErrMonadic;
use cgp::extra::monad::providers::PipeMonadic;
impl DelegateComponent<ComputerComponent> for App {
    type Delegate = PipeMonadic<ErrMonadic, Product![Double, Double]>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerComponent, __Context__, __Params__> for App
where
    PipeMonadic<
        ErrMonadic,
        Product![Double, Double],
    >: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ComputerComponent, ((), u64)> for App {}
fn main() {}
