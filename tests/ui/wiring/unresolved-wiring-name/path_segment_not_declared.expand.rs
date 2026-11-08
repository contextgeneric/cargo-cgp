#![feature(prelude_import)]
//! An `@`-path segment naming a type that was never declared, redirected to and wired.
//!
//! `@app.Missing` keeps `Missing` as a type segment, which does not resolve, so the compiler reports
//! an `E0425` at each use, the first with a "missing a type parameter" suggestion no wiring entry
//! can take. The tool reshapes the first into `[CGP-E044]`, which names the type and says a
//! lowercase segment would be a symbol, and drops the repeat and the suggestion. The `E0275` the
//! check then reports on the lookup through the unresolved key is the entry's consequence, so it
//! drops as well.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
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
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "hello".into()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
///Provider `GreetHello` for [`Greeter`].
pub struct GreetHello;
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@app.Missing)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@app.Missing),
    >: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<Missing, __Wildcard__>>> for App {
    type Delegate = GreetHello;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<Missing, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    GreetHello: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<Missing, __Wildcard__>>,
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
