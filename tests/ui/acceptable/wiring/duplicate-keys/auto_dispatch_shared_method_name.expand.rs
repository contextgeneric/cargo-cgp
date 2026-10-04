#![feature(prelude_import)]
//! Two `#[cgp_auto_dispatch]` traits in one module that both declare a method
//! named `area`. The macro names the per-variant helper `__compute_area__` and
//! the computer `ComputeArea` after the method alone, so the module defines each
//! twice: `E0428` on both names, then `E0119` on the computers' conflicting
//! impls. This is a known defect of the macro: folding the trait name into the
//! generated names would remove the clash, at the cost of renaming the public
//! `Compute{Method}` provider. The `ComputeArea` carets fall on the second `area`
//! method name, since the computer's name is spanned on the method identifier
//! it derives from; the helper's carets cover the whole attribute.
//!
//! Error class: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md, with the name clash itself
//! under https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/error_codes/e0428.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasArea {
    fn area(&self) -> f64;
}
impl<__Variants__> HasArea for __Variants__
where
    MatchWithValueHandlersRef<
        ComputeArea,
    >: for<'__a__> Computer<(), (), &'__a__ __Variants__, Output = f64>,
    __Variants__: HasExtractor,
{
    fn area(&self) -> f64 {
        <MatchWithValueHandlersRef<
            ComputeArea,
        > as Computer<_, _, _>>::compute(&(), ::core::marker::PhantomData::<()>, self)
    }
}
fn __compute_area__<'__a__, __Variants__: HasArea>(
    __Variants__: &'__a__ __Variants__,
) -> f64 {
    __Variants__.area()
}
impl<
    '__a__,
    __Variants__: HasArea,
    __Context__,
    __Code__,
> Computer<__Context__, __Code__, (&'__a__ __Variants__)> for ComputeArea {
    type Output = f64;
    fn compute(
        _context: &__Context__,
        _code: PhantomData<__Code__>,
        (arg_0): (&'__a__ __Variants__),
    ) -> Self::Output {
        __compute_area__(arg_0)
    }
}
impl<
    '__a__,
    __Variants__: HasArea,
    __Context__,
    __Code__,
> IsProviderFor<ComputerComponent, __Context__, (__Code__, (&'__a__ __Variants__))>
for ComputeArea {}
pub struct ComputeArea;
impl DelegateComponent<ComputerRefComponent> for ComputeArea
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
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for ComputeArea
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
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for ComputeArea
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
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for ComputeArea
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
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for ComputeArea
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
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for ComputeArea
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
> IsProviderFor<HandlerComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for ComputeArea
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
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
pub trait HasScaledArea {
    fn area(&self) -> f64;
}
impl<__Variants__> HasScaledArea for __Variants__
where
    MatchWithValueHandlersRef<
        ComputeArea,
    >: for<'__a__> Computer<(), (), &'__a__ __Variants__, Output = f64>,
    __Variants__: HasExtractor,
{
    fn area(&self) -> f64 {
        <MatchWithValueHandlersRef<
            ComputeArea,
        > as Computer<_, _, _>>::compute(&(), ::core::marker::PhantomData::<()>, self)
    }
}
fn __compute_area__<'__a__, __Variants__: HasScaledArea>(
    __Variants__: &'__a__ __Variants__,
) -> f64 {
    __Variants__.area()
}
impl<
    '__a__,
    __Variants__: HasScaledArea,
    __Context__,
    __Code__,
> Computer<__Context__, __Code__, (&'__a__ __Variants__)> for ComputeArea {
    type Output = f64;
    fn compute(
        _context: &__Context__,
        _code: PhantomData<__Code__>,
        (arg_0): (&'__a__ __Variants__),
    ) -> Self::Output {
        __compute_area__(arg_0)
    }
}
impl<
    '__a__,
    __Variants__: HasScaledArea,
    __Context__,
    __Code__,
> IsProviderFor<ComputerComponent, __Context__, (__Code__, (&'__a__ __Variants__))>
for ComputeArea {}
pub struct ComputeArea;
impl DelegateComponent<ComputerRefComponent> for ComputeArea
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
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for ComputeArea
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
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for ComputeArea
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
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for ComputeArea
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
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for ComputeArea
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
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for ComputeArea
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
> IsProviderFor<HandlerComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for ComputeArea
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
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
fn main() {}
