#![feature(prelude_import)]
//! A `#[cgp_auto_dispatch]` trait method taking a `Cow<str>` with its lifetime elided.
//!
//! `E0106` suggests the lifetime the macro generated, a reserved name the program cannot write; the
//! `E0726` beside it suggests the right `'_`. The macro cannot name a lifetime a path hides, since
//! only the type's definition shows it, so the compiler reports the elision; the tool drops the
//! wrong help. This header must not spell the generated name, or the source would write it.
//!
//! CGP error class: none yet; an acceptable failure of what a macro lowers, outside the catalog.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::borrow::Cow;
use cgp::prelude::*;
pub trait CanLabel {
    fn label(&self, prefix: Cow<str>) -> String;
}
impl<__Variants__> CanLabel for __Variants__
where
    MatchFirstWithValueHandlersRef<
        ComputeLabel,
    >: for<'__a__> Computer<(), (), (&'__a__ __Variants__, (Cow<str>)), Output = String>,
    __Variants__: HasExtractor,
{
    fn label(&self, arg_0: Cow<str>) -> String {
        <MatchFirstWithValueHandlersRef<
            ComputeLabel,
        > as Computer<
            _,
            _,
            _,
        >>::compute(&(), ::core::marker::PhantomData::<()>, (self, (arg_0)))
    }
}
fn __compute_label__<'__a__, __Variants__: CanLabel>(
    __Variants__: &'__a__ __Variants__,
    (arg_0): (Cow<str>),
) -> String {
    __Variants__.label(arg_0)
}
impl<
    '__a__,
    __Variants__: CanLabel,
    __Context__,
    __Code__,
> Computer<__Context__, __Code__, (&'__a__ __Variants__, (Cow<str>))> for ComputeLabel {
    type Output = String;
    fn compute(
        _context: &__Context__,
        _code: ::core::marker::PhantomData<__Code__>,
        (arg_0, arg_1): (&'__a__ __Variants__, (Cow<str>)),
    ) -> Self::Output {
        __compute_label__(arg_0, arg_1)
    }
}
impl<
    '__a__,
    __Variants__: CanLabel,
    __Context__,
    __Code__,
> IsProviderFor<
    ComputerComponent,
    __Context__,
    (__Code__, (&'__a__ __Variants__, (Cow<str>))),
> for ComputeLabel {}
pub struct ComputeLabel;
impl DelegateComponent<ComputerRefComponent> for ComputeLabel
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
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for ComputeLabel
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
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for ComputeLabel
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
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for ComputeLabel
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
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for ComputeLabel
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
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for ComputeLabel
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
> IsProviderFor<HandlerComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for ComputeLabel
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
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
fn main() {}
