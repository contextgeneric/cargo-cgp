#![feature(prelude_import)]
//! Acceptable failure: a context that joins a namespace with `namespace N;`
//! cannot also wire, directly on itself, a path that `N` already registers.
//!
//! `GreetHello` registers the path `@app.GreeterComponent` into `AppNamespace`
//! with `#[default_impl]`, so `PathCons<app, GreeterComponent>` implements
//! `AppNamespace<_>`. The `namespace AppNamespace;` header then emits a blanket
//! `impl<Key> DelegateComponent<Key> for App where Key: AppNamespace<App>`, which
//! already covers that path. The extra `@app.GreeterComponent: GreetBye` entry
//! emits a second `DelegateComponent<PathCons<app, GreeterComponent>> for App`,
//! and the two overlap — E0119. CGP lowers both entries faithfully; only the whole
//! program reveals the overlap, so it defers to the compiler.
//!
//! The rule this pins: override a component the namespace routes by shadowing its
//! *marker* only when the namespace does not itself terminate the redirect path,
//! or wire the override on a path the namespace never registers. A namespace that
//! registers the leaf path leaves nothing for the context to override there.
//!
//! This is the context-level (join) shape of the override-conflict class; contrast
//! the namespace-level (inheritance) shape in inherited_override_conflict.rs, where
//! a child namespace tries to override an entry its parent binds.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md.
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
impl<__Components__> AppNamespace<__Components__> for Path!(@app.GreeterComponent) {
    type Delegate = GreetHello;
}
impl<__Context__> Greeter<__Context__> for GreetBye {
    fn greet(__context__: &__Context__) -> String {
        "Bye".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetBye {}
pub struct GreetBye;
pub struct __AppNamespaceComponents;
pub trait AppNamespace<__Table__> {
    type Delegate;
}
impl<__Table__, __Key__, __Value__> AppNamespace<__Table__> for __Key__
where
    __Key__: DefaultNamespace<__AppNamespaceComponents>,
    __Key__: DefaultNamespace<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
pub struct App;
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: AppNamespace<App, Delegate = __Value__>,
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
    __Key__: AppNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>>
for App {
    type Delegate = GreetBye;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    GreetBye: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
