#![feature(prelude_import)]
//! A delegation whose constrained key fails on CGP's empty sum, `Sum![]`.
//!
//! The sibling of `constrained_delegate_key` for variants: `PickFirstVariantProvider<List>`
//! delegates every component only when `List: PickFirstVariant` holds, which is implemented for a
//! non-empty `Either` and never for the empty `Void`. `App` wires `GreeterComponent` to
//! `PickFirstVariantProvider<Sum![]>`, so the delegation's own bound is unmet. rustc prints the
//! empty sum as `Void`; the tool's tree and headline write it `Sum![]`, as the program does.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::prelude::*;
pub trait CanGreet {
    fn greet(&self) -> String;
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) -> String {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__)
    }
}
pub struct GreeterComponent;
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    fn greet(__context__: &__Context__) -> String {
        __Context__::greet(__context__)
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for UseContext
where
    __Context__: CanGreet,
{}
impl<__Context__, __Components__, __Path__> Greeter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::greet(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<GreeterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<GreeterComponent, __Context__, ()>
        + Greeter<__Context__>,
{}
impl<__Context__> Greeter<__Context__> for HelloGreeter {
    fn greet(__context__: &__Context__) -> String {
        "hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for HelloGreeter {}
pub struct HelloGreeter;
/// A dispatcher provider that forwards a component to whatever provider its `List` reduces to.
pub struct PickFirstVariantProvider<List>(pub PhantomData<List>);
/// The reduction: a non-empty sum yields its head provider. `Void` has no impl, so an empty sum
/// cannot reduce.
pub trait PickFirstVariant {
    type Provider;
}
impl<Head, Tail> PickFirstVariant for Either<Head, Tail> {
    type Provider = Head;
}
impl<
    Component,
    Provider,
    List: PickFirstVariant<Provider = Provider>,
> DelegateComponent<Component> for PickFirstVariantProvider<List> {
    type Delegate = Provider;
}
impl<
    Component,
    Provider,
    List: PickFirstVariant<Provider = Provider>,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<Component, __Context__, __Params__> for PickFirstVariantProvider<List>
where
    Provider: IsProviderFor<Component, __Context__, __Params__>,
{}
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = PickFirstVariantProvider<Void>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App
where
    PickFirstVariantProvider<
        Void,
    >: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
fn main() {}
