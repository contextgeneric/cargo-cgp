#![feature(prelude_import)]
//! A provider needing two prefixed components a context leaves unwired, the second with a near
//! miss among the paths the context does wire.
//!
//! `GreetWithName` requires `CanGreet` and `CanName`, both registered under `@app`. `App` wires
//! neither `@app.GreeterComponent` nor `@app.NamerComponent`, but wires `@App.NamerComponent`, a
//! slip of the second. Both causes are missing wiring on `App`, so their fixes merge into one
//! `help` at the first cause's place, and the near miss of the second cause qualifies that merged
//! fix, so it comes before it, as a near miss comes before its cause's fix.
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
///Component marker for [`CanName`].
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
pub trait CanWelcome {
    fn welcome(&self) -> String;
}
impl<__Context__> CanWelcome for __Context__
where
    __Context__: Welcomer<__Context__>,
{
    fn welcome(&self) -> String {
        __Context__::welcome(self)
    }
}
pub trait Welcomer<__Context__>: IsProviderFor<WelcomerComponent, __Context__, ()> {
    fn welcome(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Welcomer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<WelcomerComponent>
        + IsProviderFor<WelcomerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        WelcomerComponent,
    >>::Delegate: Welcomer<__Context__>,
{
    fn welcome(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            WelcomerComponent,
        >>::Delegate::welcome(__context__)
    }
}
///Component marker for [`CanWelcome`].
pub struct WelcomerComponent;
impl<__Context__> Welcomer<__Context__> for UseContext
where
    __Context__: CanWelcome,
{
    fn welcome(__context__: &__Context__) -> String {
        __Context__::welcome(__context__)
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
    fn welcome(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::welcome(__context__)
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
impl<__Context__> Welcomer<__Context__> for GreetWithName
where
    __Context__: CanGreet + CanName,
{
    fn welcome(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(
                format_args!("{0} {1}", __context__.greet(), __context__.name()),
            )
        })
    }
}
impl<__Context__> IsProviderFor<WelcomerComponent, __Context__, ()> for GreetWithName
where
    __Context__: CanGreet + CanName,
{}
///Provider `GreetWithName` for [`Welcomer`].
pub struct GreetWithName;
impl<__Context__> Namer<__Context__> for NameFixed {
    fn name(__context__: &__Context__) -> String {
        "app".to_owned()
    }
}
impl<__Context__> IsProviderFor<NamerComponent, __Context__, ()> for NameFixed {}
///Provider `NameFixed` for [`Namer`].
pub struct NameFixed;
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
    __Params__: ?Sized,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl DelegateComponent<WelcomerComponent> for App {
    type Delegate = GreetWithName;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<WelcomerComponent, __Context__, __Params__> for App
where
    GreetWithName: IsProviderFor<WelcomerComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<App, PathCons<NamerComponent, __Wildcard__>>> for App {
    type Delegate = NameFixed;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<App, PathCons<NamerComponent, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    NameFixed: IsProviderFor<
        PathCons<App, PathCons<NamerComponent, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<WelcomerComponent, ()> for App {}
fn main() {}
