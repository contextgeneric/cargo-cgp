#![feature(prelude_import)]
//! A plain type wired where a provider belongs, for a component with a lifetime parameter.
//!
//! `NotAGreeter` has no `Greeter` impl, so a call through the wiring fails on its missing provider
//! trait, as it does for a component without parameters. The provider trait takes the component's
//! lifetime ahead of the context, so a reader that took the context by raw position would land on
//! the lifetime; the tool reads it by type position and names `NotAGreeter` as no provider of
//! `Greeter`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanGreet<'a> {
    fn greet(&self, other: &'a str) -> String;
}
impl<'a, __Context__> CanGreet<'a> for __Context__
where
    __Context__: Greeter<'a, __Context__>,
{
    fn greet(&self, other: &'a str) -> String {
        __Context__::greet(self, other)
    }
}
pub trait Greeter<
    'a,
    __Context__,
>: IsProviderFor<GreeterComponent, __Context__, (Life<'a>)> {
    fn greet(__context__: &__Context__, other: &'a str) -> String;
}
impl<'a, __Provider__, __Context__> Greeter<'a, __Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, (Life<'a>)>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<'a, __Context__>,
{
    fn greet(__context__: &__Context__, other: &'a str) -> String {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__, other)
    }
}
pub struct GreeterComponent;
impl<'a, __Context__> Greeter<'a, __Context__> for UseContext
where
    __Context__: CanGreet<'a>,
{
    fn greet(__context__: &__Context__, other: &'a str) -> String {
        __Context__::greet(__context__, other)
    }
}
impl<'a, __Context__> IsProviderFor<GreeterComponent, __Context__, (Life<'a>)>
for UseContext
where
    __Context__: CanGreet<'a>,
{}
impl<'a, __Context__, __Components__, __Path__> Greeter<'a, __Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<'a, __Context__>,
{
    fn greet(__context__: &__Context__, other: &'a str) -> String {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::greet(__context__, other)
    }
}
impl<
    'a,
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<GreeterComponent, __Context__, (Life<'a>)>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<'a, __Context__>,
{}
pub struct NotAGreeter;
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = NotAGreeter;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App
where
    NotAGreeter: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {
    let app = App;
    let _ = app.greet("Ada");
}
