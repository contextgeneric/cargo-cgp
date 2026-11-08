#![feature(prelude_import)]
//! A wiring value that names no provider, beside a component whose wiring loops and whose consumer
//! trait's name starts with the first component's.
//!
//! `GreeterComponent: GreetHi` names no type, so rustc reports an `E0425` at the value, and the
//! entry leaves `Person: CanGreet` unprovable. `TwiceGreeterComponent: UseContext` leads back to
//! itself, so the call `Person.greet_twice()` overflows on `Person: CanGreetTwice`, a requirement
//! that begins with `Person: CanGreet` but is not the one the first entry leaves unprovable. The tool
//! reports `[CGP-E043]` at the value and keeps the overflow, reshaped as the cycle it is.
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
pub trait CanGreetTwice {
    fn greet_twice(&self);
}
impl<__Context__> CanGreetTwice for __Context__
where
    __Context__: TwiceGreeter<__Context__>,
{
    fn greet_twice(&self) {
        __Context__::greet_twice(self)
    }
}
pub trait TwiceGreeter<
    __Context__,
>: IsProviderFor<TwiceGreeterComponent, __Context__, ()> {
    fn greet_twice(__context__: &__Context__);
}
impl<__Provider__, __Context__> TwiceGreeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<TwiceGreeterComponent>
        + IsProviderFor<TwiceGreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        TwiceGreeterComponent,
    >>::Delegate: TwiceGreeter<__Context__>,
{
    fn greet_twice(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            TwiceGreeterComponent,
        >>::Delegate::greet_twice(__context__)
    }
}
///Component marker for [`CanGreetTwice`].
pub struct TwiceGreeterComponent;
impl<__Context__> TwiceGreeter<__Context__> for UseContext
where
    __Context__: CanGreetTwice,
{
    fn greet_twice(__context__: &__Context__) {
        __Context__::greet_twice(__context__)
    }
}
impl<__Context__> IsProviderFor<TwiceGreeterComponent, __Context__, ()> for UseContext
where
    __Context__: CanGreetTwice,
{}
impl<__Context__, __Components__, __Path__> TwiceGreeter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: TwiceGreeter<__Context__>,
{
    fn greet_twice(__context__: &__Context__) {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::greet_twice(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<TwiceGreeterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<TwiceGreeterComponent, __Context__, ()>
        + TwiceGreeter<__Context__>,
{}
pub struct Person;
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetHi;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    GreetHi: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TwiceGreeterComponent> for Person {
    type Delegate = UseContext;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<TwiceGreeterComponent, __Context__, __Params__> for Person
where
    UseContext: IsProviderFor<TwiceGreeterComponent, __Context__, __Params__>,
{}
fn main() {
    Person.greet_twice();
}
