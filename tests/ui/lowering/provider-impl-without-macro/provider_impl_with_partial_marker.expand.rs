#![feature(prelude_import)]
//! A provider-trait impl written by hand beside an `IsProviderFor` impl that covers fewer contexts.
//!
//! `GreetHello` implements `Greeter` for every `Context` but `IsProviderFor` only for `App`, so the
//! compiler reports `GreetHello: IsProviderFor<GreeterComponent, Context>` unmet at the impl's `Self`
//! type. The provider has an `IsProviderFor` impl for the component, so this is no impl written
//! without `#[cgp_provider]`, and the tool does not report it as one: the error falls to the
//! `[CGP-E002]` text rewrite, which calls the impl's parameter `Context` a context, beside rustc's
//! suggestion of a `where` bound.
//!
//! CGP error class: provider impl without the macro (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/provider-impl-without-macro.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanGreet {
    fn greet(&self) -> String;
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) -> String {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
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
    fn greet(__context__: &__Context__) -> String {
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
    fn greet(__context__: &__Context__) -> String {
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
pub struct GreetHello;
impl<Context> Greeter<Context> for GreetHello {
    fn greet(_context: &Context) -> String {
        "hello".into()
    }
}
pub struct App;
impl IsProviderFor<GreeterComponent, App, ()> for GreetHello {}
fn main() {}
