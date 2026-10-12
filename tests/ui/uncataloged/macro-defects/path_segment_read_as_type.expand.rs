#![feature(prelude_import)]
//! An `@`-path segment that `Path!` reads as a type rather than a symbol.
//!
//! `@app.f` lowers `f` to a type, a documented `Path!` defect, so rustc reports three `E0425`s, two
//! at one caret, which the tool merges into one. The overflow that follows keeps rustc's `E0275`
//! wording: the context's wiring holds the unresolved type, so the tool does not report it as a
//! `[CGP-E020]` cycle. Nothing names the `Path!` rule as the cause.
//!
//! CGP error class: none; the `Path!` segment rule is a known issue of the macro.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
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
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "hello".into()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = RedirectLookup<App, PathCons<Symbol!("app"), PathCons<f, Nil>>>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        PathCons<Symbol!("app"), PathCons<f, Nil>>,
    >: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl<__Wildcard__> DelegateComponent<PathCons<Symbol!("app"), PathCons<f, __Wildcard__>>>
for App {
    type Delegate = GreetHello;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<f, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    GreetHello: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<f, __Wildcard__>>,
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
