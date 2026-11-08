#![feature(prelude_import)]
//! `PipeHandlers::<Product![…]>::compute(…)` called directly over stages returning `Result`.
//!
//! The provider-call anchor seeds the pipeline's `Computer` from the call, and the walk reaches the
//! second `Increment`, given the first one's `Result<u8, &str>` output where it takes `u8`:
//! ``[CGP-E127] `Increment` implements `Computer` only with `Input = u8`, not `Input = Result<u8, &str>` ``.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::extra::handler::{Computer, PipeHandlers};
use cgp::prelude::*;
pub fn increment(value: u8) -> Result<u8, &'static str> {
    value.checked_add(1).ok_or("overflow")
}
impl<__Context__, __Code__> Computer<__Context__, __Code__, (u8)> for Increment {
    type Output = Result<u8, &'static str>;
    fn compute(
        _context: &__Context__,
        _code: ::core::marker::PhantomData<__Code__>,
        (arg_0): (u8),
    ) -> Self::Output {
        increment(arg_0)
    }
}
impl<
    __Context__,
    __Code__,
> IsProviderFor<ComputerComponent, __Context__, (__Code__, (u8))> for Increment {}
///Provider `Increment` for [`Computer`].
pub struct Increment;
impl DelegateComponent<ComputerRefComponent> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<ComputerRefComponent>,
{
    type Delegate = <PromoteTryComputer<
        Self,
    > as DelegateComponent<ComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<TryComputerComponent>,
{
    type Delegate = <PromoteTryComputer<
        Self,
    > as DelegateComponent<TryComputerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<TryComputerRefComponent>,
{
    type Delegate = <PromoteTryComputer<
        Self,
    > as DelegateComponent<TryComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<AsyncComputerComponent>,
{
    type Delegate = <PromoteTryComputer<
        Self,
    > as DelegateComponent<AsyncComputerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
{
    type Delegate = <PromoteTryComputer<
        Self,
    > as DelegateComponent<AsyncComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<HandlerComponent>,
{
    type Delegate = <PromoteTryComputer<
        Self,
    > as DelegateComponent<HandlerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerComponent, __Context__, __Params__> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<HandlerRefComponent>,
{
    type Delegate = <PromoteTryComputer<
        Self,
    > as DelegateComponent<HandlerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for Increment
where
    PromoteTryComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
fn main() {
    let _ = PipeHandlers::<
        Product![Increment, Increment],
    >::compute(&(), PhantomData::<()>, 1u8);
}
