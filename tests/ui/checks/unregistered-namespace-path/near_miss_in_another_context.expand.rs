#![feature(prelude_import)]
//! Two contexts miss `@app.GreeterComponent`, and only `Other` wires the near miss
//! `@App.GreeterComponent`.
//!
//! Each block is `[CGP-E113]` on `@app.GreeterComponent`. `Other`'s carries a `help` naming its
//! near-miss entry; `App`'s carries none, since a near miss is looked for in the missing key's own
//! context, and `App` wires no similar key.
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
    __Params__: ?Sized,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
pub struct Other;
impl<__Key__, __Value__> DelegateComponent<__Key__> for Other
where
    __Key__: DefaultNamespace<Other, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<
    __Key__,
    __Value__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<__Key__, __Context__, __Params__> for Other
where
    __Key__: DefaultNamespace<Other, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<App, PathCons<GreeterComponent, __Wildcard__>>> for Other {
    type Delegate = GreetHello;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<App, PathCons<GreeterComponent, __Wildcard__>>,
    __Context__,
    __Params__,
> for Other
where
    GreetHello: IsProviderFor<
        PathCons<App, PathCons<GreeterComponent, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
trait __CheckOther<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckOther<GreeterComponent, ()> for Other {}
fn main() {}
