#![feature(prelude_import)]
//! Acceptable failure: a missing entry in an aggregate provider that joins a namespace. The context
//! `App` joins `DefaultNamespace` and forwards the whole `@app` path to the aggregate provider
//! `AppComponents`, which is keyed by paths and so joins `DefaultNamespace` itself. `AppComponents`
//! binds `@app.GreeterComponent` but not `@app.NamerComponent`, so the check on `NamerComponent`
//! bottoms out on the aggregate's own namespace lookup,
//! `@app.NamerComponent: DefaultNamespace<AppComponents>`.
//!
//! The tree names the table each lookup runs in: the context's namespace hop reads `in App`, the
//! aggregate's reads `in AppComponents`, and the leaf is the `[CGP-E110]` provider-table leaf naming
//! `AppComponents`, so the reader adds the entry to the aggregate rather than to the context.
//!
//! See cgp-knowledge-base/cgp/errors/checks/unregistered-namespace-path.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E104, CGP-E110).
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
pub trait CanName {
    fn name(&self) -> String;
}
impl<__Context__> CanName for __Context__
where
    __Context__: Namer<__Context__>,
{
    fn name(&self) -> String {
        __Context__::name(self)
    }
}
pub trait Namer<__Context__>: IsProviderFor<NamerComponent, __Context__, ()> {
    fn name(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Namer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<NamerComponent>
        + IsProviderFor<NamerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<NamerComponent>>::Delegate: Namer<__Context__>,
{
    fn name(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<NamerComponent>>::Delegate::name(__context__)
    }
}
pub struct NamerComponent;
impl<__Context__> Namer<__Context__> for UseContext
where
    __Context__: CanName,
{
    fn name(__context__: &__Context__) -> String {
        __Context__::name(__context__)
    }
}
impl<__Context__> IsProviderFor<NamerComponent, __Context__, ()> for UseContext
where
    __Context__: CanName,
{}
impl<__Context__, __Components__, __Path__> Namer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Namer<__Context__>,
{
    fn name(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::name(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<NamerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<NamerComponent, __Context__, ()> + Namer<__Context__>,
{}
impl<__Components__> DefaultNamespace<__Components__> for NamerComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@app.NamerComponent)>;
}
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "Hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
pub struct AppComponents;
impl<__Key__, __Value__> DelegateComponent<__Key__> for AppComponents
where
    __Key__: DefaultNamespace<AppComponents, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<
    __Key__,
    __Value__,
    __Context__,
    __Params__,
> IsProviderFor<__Key__, __Context__, __Params__> for AppComponents
where
    __Key__: DefaultNamespace<AppComponents, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>>
for AppComponents {
    type Delegate = GreetHello;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>,
    __Context__,
    __Params__,
> for AppComponents
where
    GreetHello: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
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
impl<__Wildcard__> DelegateComponent<PathCons<Symbol!("app"), __Wildcard__>> for App {
    type Delegate = AppComponents;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<PathCons<Symbol!("app"), __Wildcard__>, __Context__, __Params__> for App
where
    AppComponents: IsProviderFor<
        PathCons<Symbol!("app"), __Wildcard__>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<NamerComponent, ()> for App {}
fn main() {}
