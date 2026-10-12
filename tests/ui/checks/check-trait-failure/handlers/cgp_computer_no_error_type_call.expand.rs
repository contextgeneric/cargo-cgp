#![feature(prelude_import)]
//! `try_compute` on a context wiring a `#[cgp_computer]` provider and no error type.
//!
//! The one root cause is `[CGP-E113]` on `ErrorTypeProviderComponent`. The call-site anchor types
//! the input `(_, _)`, so `Add`'s own `Computer` impl does not match and the walk reaches its
//! delegation blanket; `Add` has its own impl of the trait, so the unmet `DelegateComponent` there
//! is a dead end rather than a missing dispatch entry, even though `Add` wires its promotions.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::extra::handler::CanTryCompute;
use cgp::prelude::*;
fn add(a: u64, b: u64) -> u64 {
    a + b
}
impl<__Context__, __Code__> Computer<__Context__, __Code__, (u64, u64)> for Add {
    type Output = u64;
    fn compute(
        _context: &__Context__,
        _code: ::core::marker::PhantomData<__Code__>,
        (arg_0, arg_1): (u64, u64),
    ) -> Self::Output {
        add(arg_0, arg_1)
    }
}
impl<
    __Context__,
    __Code__,
> IsProviderFor<ComputerComponent, __Context__, (__Code__, (u64, u64))> for Add {}
pub struct Add;
impl DelegateComponent<ComputerRefComponent> for Add
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<ComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for Add
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for Add
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<TryComputerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for Add
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for Add
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<TryComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for Add
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for Add
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<AsyncComputerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for Add
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for Add
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<AsyncComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for Add
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for Add
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<HandlerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerComponent, __Context__, __Params__> for Add
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for Add
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<HandlerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for Add
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
pub struct App;
impl DelegateComponent<TryComputerComponent> for App {
    type Delegate = Add;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for App
where
    Add: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
fn main() {
    let _ = App.try_compute(PhantomData::<()>, (1, 2));
}
