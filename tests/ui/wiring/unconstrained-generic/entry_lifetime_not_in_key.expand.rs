#![feature(prelude_import)]
//! A per-entry lifetime that only the provider uses: `<'a> GreeterComponent: GreetRef<'a>`. The
//! entry lowers to an impl whose associated `Delegate` names `'a`, so the lifetime is unconstrained
//! (E0207).
//!
//! The tool leaves rustc's error and its suggestion to add the lifetime to `Person`, which does not
//! compile. `[CGP-E019]` lists only type and const parameters, since a lifetime the key leaves out is
//! legal unless an associated type uses it; the shape is an open issue of the class.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/unconstrained-generic.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
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
pub struct GreetRef<'a>(pub PhantomData<&'a ()>);
impl<'a, Context> Greeter<Context> for GreetRef<'a> {
    fn greet(_context: &Context) {}
}
impl<'a, Context> IsProviderFor<GreeterComponent, Context, ()> for GreetRef<'a> {}
pub struct Person;
impl<'a> DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetRef<'a>;
}
impl<
    'a,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    GreetRef<'a>: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {}
