#![feature(prelude_import)]
//! A wiring value that misspells a provider of the key's component, beside a closer name that
//! provides another component.
//!
//! `GreeterComponent: GreetHelo` names no type, so rustc reports an `E0425` and suggests a
//! similarly named item. The tool reports `[CGP-E043]`, and its help names the provider of
//! `Greeter` with a similar name, `GreetHello`. `Greethelo` matches the misspelling more closely,
//! ignoring case, but it provides `Farewell`, so it is not offered.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanGreet {
    fn greet(&self);
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__);
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__)
    }
}
///Component marker for [`CanGreet`].
pub struct GreeterComponent;
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    fn greet(__context__: &__Context__) {
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
    fn greet(__context__: &__Context__) {
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
pub trait CanSayBye {
    fn say_bye(&self);
}
impl<__Context__> CanSayBye for __Context__
where
    __Context__: Farewell<__Context__>,
{
    fn say_bye(&self) {
        __Context__::say_bye(self)
    }
}
pub trait Farewell<__Context__>: IsProviderFor<FarewellComponent, __Context__, ()> {
    fn say_bye(__context__: &__Context__);
}
impl<__Provider__, __Context__> Farewell<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<FarewellComponent>
        + IsProviderFor<FarewellComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        FarewellComponent,
    >>::Delegate: Farewell<__Context__>,
{
    fn say_bye(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            FarewellComponent,
        >>::Delegate::say_bye(__context__)
    }
}
///Component marker for [`CanSayBye`].
pub struct FarewellComponent;
impl<__Context__> Farewell<__Context__> for UseContext
where
    __Context__: CanSayBye,
{
    fn say_bye(__context__: &__Context__) {
        __Context__::say_bye(__context__)
    }
}
impl<__Context__> IsProviderFor<FarewellComponent, __Context__, ()> for UseContext
where
    __Context__: CanSayBye,
{}
impl<__Context__, __Components__, __Path__> Farewell<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Farewell<__Context__>,
{
    fn say_bye(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::say_bye(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<FarewellComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<FarewellComponent, __Context__, ()>
        + Farewell<__Context__>,
{}
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
///Provider `GreetHello` for [`Greeter`].
pub struct GreetHello;
impl<__Context__> Farewell<__Context__> for Greethelo {
    fn say_bye(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<FarewellComponent, __Context__, ()> for Greethelo {}
///Provider `Greethelo` for [`Farewell`].
pub struct Greethelo;
pub struct Person;
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetHelo;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    GreetHelo: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl DelegateComponent<FarewellComponent> for Person {
    type Delegate = Greethelo;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<FarewellComponent, __Context__, __Params__> for Person
where
    Greethelo: IsProviderFor<FarewellComponent, __Context__, __Params__>,
{}
fn main() {}
