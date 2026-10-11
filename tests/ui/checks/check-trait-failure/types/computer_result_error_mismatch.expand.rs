#![feature(prelude_import)]
//! A `#[cgp_computer]` function returning `Result<u64, String>`, called through
//! `try_compute` on a context whose abstract error type is `()` rather than
//! `String`. The fallible promotion passes the function's `Err` through
//! unconverted, so it requires the context's error type to equal the function's,
//! and the call fails with `E0271`. The note names both `Result` types and points
//! at the function's return type, so the mismatch is stated plainly.
//!
//! Error class: an associated-type mismatch,
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/error_codes/e0271.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::TryComputer;
use cgp::prelude::*;
fn checked_add(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b).ok_or_else(|| "overflow".to_owned())
}
impl<__Context__, __Code__> Computer<__Context__, __Code__, (u64, u64)> for CheckedAdd {
    type Output = Result<u64, String>;
    fn compute(
        _context: &__Context__,
        _code: ::core::marker::PhantomData<__Code__>,
        (arg_0, arg_1): (u64, u64),
    ) -> Self::Output {
        checked_add(arg_0, arg_1)
    }
}
impl<
    __Context__,
    __Code__,
> IsProviderFor<ComputerComponent, __Context__, (__Code__, (u64, u64))> for CheckedAdd {}
pub struct CheckedAdd;
impl DelegateComponent<ComputerRefComponent> for CheckedAdd
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
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for CheckedAdd
where
    PromoteTryComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for CheckedAdd
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
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for CheckedAdd
where
    PromoteTryComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for CheckedAdd
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
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for CheckedAdd
where
    PromoteTryComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for CheckedAdd
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
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for CheckedAdd
where
    PromoteTryComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for CheckedAdd
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
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for CheckedAdd
where
    PromoteTryComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for CheckedAdd
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
> IsProviderFor<HandlerComponent, __Context__, __Params__> for CheckedAdd
where
    PromoteTryComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for CheckedAdd
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
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for CheckedAdd
where
    PromoteTryComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteTryComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
pub struct App;
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<()>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<()>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
fn main() {
    let _ = CheckedAdd::try_compute(&App, PhantomData::<()>, (1, 2));
}
