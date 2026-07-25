#![feature(prelude_import)]
//! Acceptable failure: a per-entry generic list on a `delegate_components!`
//! mapping whose parameter appears only in the *provider value* and not in the
//! *key*. The macro faithfully lowers `<T> GreeterComponent: GreetWith<T>` into
//! `impl<T> DelegateComponent<GreeterComponent> for Person { type Delegate =
//! GreetWith<T>; }`, where `T` is constrained by neither the trait, the self
//! type, nor a predicate — so the compiler rejects it with E0207.
//!
//! A per-entry generic is only well-formed when it appears in the key (as in
//! `<T2> BazKey<T1, T2>: BarValue<T1>`, where `DelegateComponent<BazKey<..>>`
//! binds it). Writing one that never reaches the key is ill-formed input, and
//! the macro lowers it faithfully rather than second-guessing it — so `rustc`
//! rejects the unconstrained parameter with exactly the E0207 it would give a
//! hand-written `impl<T>` with an unused parameter. Deferring this to the
//! compiler is the intended behavior, not a macro defect.
//!
//! See docs/errors/wiring/unconstrained-generic.md.
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
pub struct Person;
impl<T> DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetWith<T>;
}
impl<T, __Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for Person
where
    GreetWith<T>: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {}
