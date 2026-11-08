#![feature(prelude_import)]
//! A context joining a namespace that does not inherit the namespace its component is prefixed into.
//!
//! `[CGP-E113]` on `GreeterComponent`, with a `help` naming the `@app.GreeterComponent` entry the
//! lookup never reaches, since only `DefaultNamespace` routes the component there.
//!
//! CGP error class: unregistered namespace path (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/unregistered-namespace-path.md).
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
impl<__Components__> DefaultNamespace<__Components__> for GreeterComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@app.GreeterComponent)>;
}
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
///Provider `GreetHello` for [`Greeter`].
pub struct GreetHello;
///Wiring table for the `AppNamespace` namespace.
pub struct __AppNamespaceComponents;
///The `AppNamespace` namespace's wiring table.
pub trait AppNamespace<__Table__> {
    ///The provider table the `AppNamespace` namespace delegates to.
    type Delegate;
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
    __Params__: ?Sized,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: AppNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>>
for App {
    type Delegate = GreetHello;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    GreetHello: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
fn main() {}
