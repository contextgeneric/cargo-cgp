#![feature(prelude_import)]
//! A `check_components!` failure whose root cause is a provider written for another context:
//! `GreetPerson` implements `Greeter` for `Person` alone, and `Robot` wires it.
//!
//! The walk reaches `GreetPerson: Greeter<Robot>`, which no impl of `GreetPerson` matches, and
//! names the context the provider takes:
//! ``[CGP-E127] `GreetPerson` implements `Greeter` only for `Person`, not `Robot` ``.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
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
pub struct Person;
pub struct Robot;
impl Greeter<Person> for GreetPerson {
    fn greet(__person__: &Person) {}
}
impl IsProviderFor<GreeterComponent, Person, ()> for GreetPerson {}
pub struct GreetPerson;
impl DelegateComponent<GreeterComponent> for Robot {
    type Delegate = GreetPerson;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Robot
where
    GreetPerson: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckRobot<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckRobot<GreeterComponent, ()> for Robot {}
fn main() {}
