#![feature(prelude_import)]
//! Acceptable failure: two entries in a *single* `delegate_components!` block
//! that map the same component key produce two conflicting
//! `DelegateComponent<GreeterComponent>` impls for `Person`, rejected with the
//! coherence error E0119 — the same failure as [duplicate_key.rs], reached with
//! one block instead of two.
//!
//! This fixture exists to pin the **error span**: each conflicting entry lowers
//! to an impl re-spanned onto its own key, so E0119 points at the two distinct
//! `GreeterComponent` tokens (the "first implementation here" note lands on the
//! first entry, the conflict caret on the second) rather than at the whole
//! block. If the per-entry re-spanning in `mapping/eval.rs` regresses, both
//! carets snap back to the macro invocation and this `.stderr` changes.
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
impl<__Context__> Greeter<__Context__> for GreetGoodbye {
    fn greet(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetGoodbye {}
pub struct GreetGoodbye;
pub struct Person;
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetHello;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for Person
where
    GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetGoodbye;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for Person
where
    GreetGoodbye: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {}
