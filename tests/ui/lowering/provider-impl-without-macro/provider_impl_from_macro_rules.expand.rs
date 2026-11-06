#![feature(prelude_import)]
//! A provider-trait impl a `macro_rules!` macro writes without `#[cgp_provider]`.
//!
//! The macro writes `impl<Context> Greeter<Context> for GreetHello` with no `IsProviderFor` impl, so
//! the compiler rejects it on the provider trait's supertrait. The `[CGP-E039]` detector takes only
//! an impl whose trait reference the program wrote, and this one comes from a macro expansion, so
//! the error falls to the text rewrite: a `[CGP-E002]` headline that calls the impl's parameter
//! `Context` a context, over rustc's list of `IsProviderFor` implementors cut to eight.
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
macro_rules! greeter_provider {
    ($provider:ident) => {
        pub struct $provider; impl < Context > Greeter < Context > for $provider { fn
        greet(_context : & Context) -> String { "hello".into() } }
    };
}
pub struct GreetHello;
impl<Context> Greeter<Context> for GreetHello {
    fn greet(_context: &Context) -> String {
        "hello".into()
    }
}
fn main() {}
