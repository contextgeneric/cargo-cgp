#![feature(prelude_import)]
//! Acceptable failure: a context joins a namespace that *routes* a prefixed
//! component to a path, but no entry ever *terminates* that path with a provider,
//! so the namespace lookup finds no delegate.
//!
//! `CanGreet` carries `#[prefix(@app in DefaultNamespace)]`, so `DefaultNamespace`
//! resolves `GreeterComponent` to `RedirectLookup<_, @app.GreeterComponent>`. `App`
//! joins `DefaultNamespace` with `namespace DefaultNamespace;`, so its
//! `GreeterComponent` lookup follows that redirect — but nothing (no `#[default_impl]`,
//! no namespace body entry, no direct `@app.GreeterComponent:` line) ever binds a
//! provider at that path. The defined `GreetHello` is never wired there. The terminal
//! failure is the namespace lookup `Path!(@app.GreeterComponent): DefaultNamespace<App>`,
//! for which there is no impl.
//!
//! This is the *lookup-failed* class — no provider is found at all — distinct from
//! an unsatisfied *dependency*, where a provider is found but its `where` clause is
//! unmet. The forgotten binding (usually a missing `#[default_impl]` or body entry)
//! is the common namespace mistake it captures. The resolver recognizes the unmet
//! namespace-lookup trait by its `Delegate`-associated-type fingerprint and words the
//! root cause as a `MissingRedirectWiring`: the redirect forwards the lookup to the path
//! in `App`, but `App` has no delegate entry for it — naming the path the programmer must
//! wire rather than leaving a raw `DefaultNamespace` bound.
//!
//! See docs/errors/checks/unregistered-namespace-path.md.
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
impl<__Components__> DefaultNamespace<__Components__> for GreeterComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@app.GreeterComponent)>;
}
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "Hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
pub struct App;
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<
    __Key__,
    __Value__,
    __Context__,
    __Params__,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
fn main() {}
