#![feature(prelude_import)]
//! Acceptable failure: a provider registers itself under a prefixed component's *bare
//! marker* in the very namespace the component's `#[prefix(...)]` registers into, so the
//! registration collides with the prefix's own redirect.
//!
//! `#[prefix(@app in DefaultNamespace)]` emits `impl<__Components__>
//! DefaultNamespace<__Components__> for GreeterComponent` whose `Delegate` is a
//! `RedirectLookup` down `@app.GreeterComponent`, and `#[default_impl(GreeterComponent in
//! DefaultNamespace)]` emits a second impl of that same trait for that same marker, so
//! coherence rejects the pair (`E0119`). Unlike namespace_inherited_unprefixed_key.rs, no
//! inheritance blanket is involved: both impls are concrete registrations for one key, and
//! the collision is direct.
//!
//! The mistake is the same one in both, though — a prefixed component is addressed by its
//! path — so the tool reads the `RedirectLookup` straight off the prefix entry's `Delegate`
//! and reports `[CGP-E007]`, naming `@app.GreeterComponent` as the key to register under.
//! Without that, the bare marker key leaves the conflict unclassified and rustc's raw
//! `conflicting implementations of trait \`DefaultNamespace<_>\`` is all the reader gets.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md,
//! cgp-knowledge-base/cgp/reference/attributes/default_impl.md (Known issues), and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E007).
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
impl<__Components__> DefaultNamespace<__Components__> for GreeterComponent {
    type Delegate = GreetHello;
}
fn main() {}
