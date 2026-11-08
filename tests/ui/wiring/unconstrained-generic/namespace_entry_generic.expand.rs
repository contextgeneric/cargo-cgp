#![feature(prelude_import)]
//! A per-entry generic that never reaches the key, in a `cgp_namespace!` entry.
//!
//! The entry lowers to an impl of the namespace's lookup trait for its key, which leaves `T`
//! unconstrained, so the compiler rejects it with `E0207` and suggests making the component
//! generic, which is not the fix. The tool reports a `[CGP-E019]` naming the namespace, with the
//! help a context's entry gets.
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
pub struct GreetWith<T>(pub PhantomData<T>);
impl<Context, T> Greeter<Context> for GreetWith<T> {
    fn greet(_context: &Context) {}
}
impl<Context, T> IsProviderFor<GreeterComponent, Context, ()> for GreetWith<T> {}
///Wiring table for the `AppNamespace` namespace.
pub struct __AppNamespaceComponents;
///The `AppNamespace` namespace's wiring table.
pub trait AppNamespace<__Table__> {
    ///The provider table the `AppNamespace` namespace delegates to.
    type Delegate;
}
impl<__Table__, T> AppNamespace<__Table__> for GreeterComponent {
    type Delegate = GreetWith<T>;
}
fn main() {}
