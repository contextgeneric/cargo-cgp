#![feature(prelude_import)]
//! A per-entry generic on a `delegate_components!` mapping that appears only in the provider, not
//! in the key.
//!
//! The macro lowers `<T> GreeterComponent: GreetWith<T>` to
//! `impl<T> DelegateComponent<GreeterComponent> for Person { type Delegate = GreetWith<T>; }` and
//! its `IsProviderFor` twin, where neither the trait, the self type, nor a predicate binds `T`, so
//! the compiler rejects `T` with `E0207` once per impl. Its two suggestions contradict each other,
//! and neither compiles: adding `T` to `Person` leaves the struct's parameter unused, and removing
//! `<T>` leaves `GreetWith<T>` naming an undeclared type. The tool reports one `[CGP-E019]` naming
//! the entry, with a help to write a concrete type in place of `T` and remove it from the entry's
//! generic parameters, which compiles.
//!
//! A per-entry generic is well formed only when it reaches the key, as in
//! `<T2> BazKey<T1, T2>: BarValue<T1>`, where `DelegateComponent<BazKey<..>>` binds it. The macro
//! lowers an entry whose generic never reaches the key as written, so the compiler rejects it as it
//! would a hand-written impl with an unused parameter; this is not a macro defect.
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
pub struct Person;
impl<T> DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetWith<T>;
}
impl<
    T,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    GreetWith<T>: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {}
