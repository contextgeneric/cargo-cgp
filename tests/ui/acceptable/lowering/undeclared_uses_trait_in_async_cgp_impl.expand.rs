#![feature(prelude_import)]
//! The async form of [`undeclared_uses_trait_in_cgp_impl`](undeclared_uses_trait_in_cgp_impl.rs):
//! a `#[cgp_impl]` provider whose `async fn` body calls a trait method without declaring the trait
//! via `#[uses]`.
//!
//! An `async fn` body does not see its parameters directly: lowering moves each one into the future
//! with an untyped `let __context__ = __context__;`, so the failing call's receiver resolves to that
//! fresh binding rather than to the parameter declared `&__Context__`. Raw rustc reports the same
//! `E0599` about `&__Context__` as the sync case. The tool follows the forwarding `let` back to the
//! parameter and reshapes it into the same `[CGP-E012]` error with the `#[uses(PersonName)]` help.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
trait PersonName {
    fn person_name(&self) -> String;
}
impl<__Context__> PersonName for __Context__
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn person_name(&self) -> String {
        let name: &str = self
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        name.to_owned()
    }
}
pub trait CanGreet {
    fn greet(&self) -> impl ::core::future::Future<Output = String>;
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    async fn greet(&self) -> String {
        __Context__::greet(self).await
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__) -> impl ::core::future::Future<Output = String>;
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    async fn greet(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__)
            .await
    }
}
pub struct GreeterComponent;
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    async fn greet(__context__: &__Context__) -> String {
        __Context__::greet(__context__).await
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
    async fn greet(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::greet(__context__)
            .await
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
impl<__Context__> Greeter<__Context__> for GreetHello {
    async fn greet(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0}!", __context__.person_name()))
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
fn main() {}
