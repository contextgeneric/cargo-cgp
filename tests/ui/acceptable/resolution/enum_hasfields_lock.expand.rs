#![feature(prelude_import)]
//! Regression: a resolution-class `E0599` near CGP wiring must not crash the resolver.
//!
//! The `GreetChoice` provider's `where` clause names `Choice::Fields` — but `Choice` is an enum
//! with no such associated item reachable that way (the writer meant `<Choice as HasFields>::Fields`
//! and forgot the qualified form), so rustc reports `E0599: no variant named Fields`. Crucially that
//! error is emitted *during* predicate lowering (`gather_explicit_predicates_of`), while that query
//! is mid-flight.
//!
//! The resolver used to treat every `E0599` as a candidate consumer-method failure and run its trait
//! solver on this one; the solver re-forced an emitting query and re-entered the already-held
//! `DiagCtxt` lock, aborting the compiler with `lock was already held`. The resolver now declines an
//! `E0599` that is not the "method exists but its trait bounds were not satisfied" shape — which is
//! both crash-safe (no solving on it) and correct, since a name-resolution error is not a CGP wiring
//! failure. rustc's own clear `E0599` passes through unchanged.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanGreet {
    fn greet(&self);
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__);
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) {
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
    fn greet(__context__: &__Context__) {
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
    fn greet(__context__: &__Context__) {
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
pub enum Choice {
    Yes,
    No,
}
impl<__Context__> Greeter<__Context__> for GreetChoice
where
    Choice::Fields: Sized,
{
    fn greet(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetChoice
where
    Choice::Fields: Sized,
{}
pub struct GreetChoice;
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetChoice;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for App
where
    GreetChoice: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {}
