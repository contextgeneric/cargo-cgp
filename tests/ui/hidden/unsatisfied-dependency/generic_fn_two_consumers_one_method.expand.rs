#![feature(prelude_import)]
//! A method two consumer traits declare, called on an unbounded type parameter.
//!
//! `CanGreet` and `CanWelcome` both declare `greet`, and `fn run<Context>` calls `context.greet()`
//! without bounding `Context` by either. Either bound would let the call resolve, and the call does
//! not say which the reader means, so the tool names neither and keeps rustc's error, which lists
//! both traits.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
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
pub trait CanWelcome {
    fn greet(&self);
}
impl<__Context__> CanWelcome for __Context__
where
    __Context__: Welcomer<__Context__>,
{
    fn greet(&self) {
        __Context__::greet(self)
    }
}
pub trait Welcomer<__Context__>: IsProviderFor<WelcomerComponent, __Context__, ()> {
    fn greet(__context__: &__Context__);
}
impl<__Provider__, __Context__> Welcomer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<WelcomerComponent>
        + IsProviderFor<WelcomerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        WelcomerComponent,
    >>::Delegate: Welcomer<__Context__>,
{
    fn greet(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            WelcomerComponent,
        >>::Delegate::greet(__context__)
    }
}
///Component marker for [`CanWelcome`].
pub struct WelcomerComponent;
impl<__Context__> Welcomer<__Context__> for UseContext
where
    __Context__: CanWelcome,
{
    fn greet(__context__: &__Context__) {
        __Context__::greet(__context__)
    }
}
impl<__Context__> IsProviderFor<WelcomerComponent, __Context__, ()> for UseContext
where
    __Context__: CanWelcome,
{}
impl<__Context__, __Components__, __Path__> Welcomer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Welcomer<__Context__>,
{
    fn greet(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::greet(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<WelcomerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<WelcomerComponent, __Context__, ()>
        + Welcomer<__Context__>,
{}
fn run<Context>(context: &Context) {
    context.greet();
}
fn main() {}
