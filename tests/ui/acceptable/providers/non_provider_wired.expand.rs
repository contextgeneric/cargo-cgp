#![feature(prelude_import)]
//! A `check_components!` failure whose root cause is a plain type wired where a *provider* was
//! expected: a higher-order provider's inner slot is filled with a struct that does not implement
//! the provider trait at all. Distilled from the `money-transfer-api` example, where an endpoint
//! wrapper `UseBasicAuth<QueryBalanceRequest>` is missing its inner `HandleQueryBalance<…>` handler,
//! so the *request* type `QueryBalanceRequest` sits where an `ApiHandler` provider belongs.
//!
//! Here `WrapGreeter<Inner>` requires `Inner: Greeter` (its inner provider), but the context wires
//! `WrapGreeter<NotAGreeter>` where `NotAGreeter` is an ordinary struct with no `Greeter` impl. The
//! walk reaches `NotAGreeter: Greeter<App>`, whose only matching impl is the CGP delegation blanket,
//! so it bottoms out on an unmet `NotAGreeter: DelegateComponent<GreeterComponent>`.
//!
//! The resolver tells this apart from a leaf-provider dead-end (a valid provider reached by an input
//! mismatch, whose real cause runs through its concrete impl) by whether the owner has a concrete
//! impl of the provider trait at all: `NotAGreeter` has *no* `Greeter` impl, so it is genuinely not
//! a provider, reported as a [`CGP-E111`] `NotAProvider` leaf — `the provider trait \`Greeter\` is
//! not implemented for \`NotAGreeter\``. Before this, the resolver dropped the leaf and declined to a
//! `[CGP-E002]` block naming the whole `WrapGreeter<NotAGreeter>` pipeline (and leaking rustc's giant
//! implementor list), with the real cause nowhere.
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
/// A real leaf provider for `Greeter`.
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
/// A higher-order provider that wraps an inner `Greeter` — the shape a wrapper endpoint has.
impl<__Context__, Inner> Greeter<__Context__> for WrapGreeter<Inner>
where
    Inner: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        Inner::greet(__context__)
    }
}
impl<__Context__, Inner> IsProviderFor<GreeterComponent, __Context__, ()>
for WrapGreeter<Inner>
where
    Inner: IsProviderFor<GreeterComponent, __Context__, ()> + Greeter<__Context__>,
{}
pub struct WrapGreeter<Inner>(pub ::core::marker::PhantomData<Inner>);
/// An ordinary struct that is **not** a `Greeter` provider — wired where a provider is expected.
pub struct NotAGreeter;
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = WrapGreeter<NotAGreeter>;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for App
where
    WrapGreeter<NotAGreeter>: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
fn main() {}
