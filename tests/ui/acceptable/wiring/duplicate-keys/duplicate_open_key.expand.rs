#![feature(prelude_import)]
//! Acceptable failure: an `open` header and an explicit mapping that both wire
//! the same component. `open GreeterComponent;` emits a `DelegateComponent<
//! GreeterComponent>` redirect impl, and the following `GreeterComponent:
//! GreetHello` emits another, so they conflict with the coherence error E0119 —
//! the third duplicate-key shape named in the Failure modes doc, alongside
//! [duplicate_key.rs] (two blocks) and [duplicate_key_same_block.rs] (two plain
//! entries).
//!
//! This fixture pins the **error span** for the `open`-header entry, whose span
//! is sourced in `statement/open.rs` from the opened component (a distinct source
//! from the plain key path): the "first implementation here" note lands on the
//! `GreeterComponent` inside `open …;`, and the conflict caret on the explicit
//! mapping's key.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md; error-span
//! mechanics in
//! cgp-knowledge-base/cgp/implementation/entrypoints/delegate_components.md.
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
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
pub struct Person;
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = RedirectLookup<Person, Path!(@GreeterComponent)>;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for Person
where
    RedirectLookup<
        Person,
        Path!(@GreeterComponent),
    >: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetHello;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for Person
where
    GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {}
