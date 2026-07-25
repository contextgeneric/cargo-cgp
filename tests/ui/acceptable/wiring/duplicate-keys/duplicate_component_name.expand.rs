#![feature(prelude_import)]
//! Acceptable failure: `#[cgp_component(Greeter)]` derives a marker
//! `pub struct GreeterComponent;`, and this module also declares its own
//! `GreeterComponent`, so the name is defined twice (E0428). `#[cgp_component]`
//! expands without any view of the rest of the module, so it emits the derived
//! marker faithfully and lets the compiler report the clash — exactly as two
//! hand-written definitions would.
//!
//! This fixture pins the span of the *derived* `#[cgp_component]` marker. The
//! E0428 "previous definition of the type `GreeterComponent` here" note falls on
//! the `Greeter` provider name the user wrote inside `#[cgp_component(…)]`, not on
//! the whole attribute, because the derived marker struct ident is emitted with
//! the provider identifier's own span (see
//! cgp-macro-core/src/types/cgp_component/args/component_args.rs). A regression
//! that stamped the marker with `Span::call_site()` would move that note onto the
//! whole `#[cgp_component(..)]` attribute — the leak the span fix removed so that
//! cross-crate go-to-definition on the marker resolves to the provider name alone.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.
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
pub struct GreeterComponent;
fn main() {}
