#![feature(prelude_import)]
//! Per-entry generics that never reach the key, in three entry shapes beside the plain one.
//!
//! An entry's generic list binds its parameters on the impls the entry lowers to, and only the key
//! can constrain them there. So the compiler rejects each parameter the key leaves out with
//! `E0207`, once per generated impl, and suggests adding it to the context or removing it. Here an
//! entry binds two such parameters, an entry binds one it never uses, and a `->` entry forwards to a
//! table named with its parameter.
//!
//! Each entry becomes one `[CGP-E019]`: the two parameters are named together, the unused one is
//! only removed, and the `->` entry's help points at where the entry uses its parameter. Each help
//! compiles.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
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
pub struct GreetWith<T>(pub PhantomData<T>);
impl<Context, T> Greeter<Context> for GreetWith<T> {
    fn greet(_context: &Context) {}
}
impl<Context, T> IsProviderFor<GreeterComponent, Context, ()> for GreetWith<T> {}
pub struct GreetPair<A, B>(pub PhantomData<(A, B)>);
impl<Context, A, B> Greeter<Context> for GreetPair<A, B> {
    fn greet(_context: &Context) {}
}
impl<Context, A, B> IsProviderFor<GreeterComponent, Context, ()> for GreetPair<A, B> {}
pub struct Table<T>(pub PhantomData<T>);
impl<T> DelegateComponent<GreeterComponent> for Table<T> {
    type Delegate = GreetWith<T>;
}
impl<
    T,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Table<T>
where
    GreetWith<T>: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
pub struct Person;
impl<T, U> DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetPair<T, U>;
}
impl<
    T,
    U,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    GreetPair<T, U>: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
pub struct Robot;
impl<T> DelegateComponent<GreeterComponent> for Robot {
    type Delegate = GreetWith<u32>;
}
impl<
    T,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Robot
where
    GreetWith<u32>: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
pub struct Kiosk;
impl<T> DelegateComponent<GreeterComponent> for Kiosk
where
    Table<T>: DelegateComponent<GreeterComponent>,
{
    type Delegate = <Table<T> as DelegateComponent<GreeterComponent>>::Delegate;
}
impl<
    T,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Kiosk
where
    Table<T>: DelegateComponent<GreeterComponent>,
    <Table<
        T,
    > as DelegateComponent<
        GreeterComponent,
    >>::Delegate: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {}
