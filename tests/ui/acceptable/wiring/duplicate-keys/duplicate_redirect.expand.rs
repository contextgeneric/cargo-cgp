#![feature(prelude_import)]
//! Acceptable failure: the same component redirected twice with explicit `=>` mappings, to two
//! different paths. Each `FooComponent => @app.foo` lowers to a `DelegateComponent<FooComponent>`
//! whose `Delegate` is a `RedirectLookup`, so the two conflict with the coherence error E0119 — a
//! *duplicate redirect*, distinct from a redirect that collides with a direct wiring
//! ([duplicate_open_key.rs]).
//!
//! The tool recognizes that *both* conflicting entries redirect the same key: it drops the
//! redundant `IsProviderFor` half and rewrites the `DelegateComponent` half to `[CGP-E008]
//! duplicate redirect for component `FooComponent` … redirected to both `@app.foo` and `@app.bar``,
//! naming both redirect targets, while keeping rustc's two carets on the `FooComponent =>` lines.
//! If the redirect detection in `resolve/conflict.rs` regresses (counting one redirect instead of
//! two), the header reverts to the single-redirect `[CGP-E007]` form.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E008).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanFoo {
    fn foo(&self);
}
impl<__Context__> CanFoo for __Context__
where
    __Context__: Foo<__Context__>,
{
    fn foo(&self) {
        __Context__::foo(self)
    }
}
pub trait Foo<__Context__>: IsProviderFor<FooComponent, __Context__, ()> {
    fn foo(__context__: &__Context__);
}
impl<__Provider__, __Context__> Foo<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<FooComponent>
        + IsProviderFor<FooComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<FooComponent>>::Delegate: Foo<__Context__>,
{
    fn foo(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<FooComponent>>::Delegate::foo(__context__)
    }
}
pub struct FooComponent;
impl<__Context__> Foo<__Context__> for UseContext
where
    __Context__: CanFoo,
{
    fn foo(__context__: &__Context__) {
        __Context__::foo(__context__)
    }
}
impl<__Context__> IsProviderFor<FooComponent, __Context__, ()> for UseContext
where
    __Context__: CanFoo,
{}
impl<__Context__, __Components__, __Path__> Foo<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Foo<__Context__>,
{
    fn foo(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::foo(__context__)
    }
}
impl<__Context__, __Components__, __Path__> IsProviderFor<FooComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<FooComponent, __Context__, ()> + Foo<__Context__>,
{}
pub struct App;
impl DelegateComponent<FooComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@app.foo)>;
}
impl<__Context__, __Params__> IsProviderFor<FooComponent, __Context__, __Params__>
for App
where
    RedirectLookup<
        App,
        Path!(@app.foo),
    >: IsProviderFor<FooComponent, __Context__, __Params__>,
{}
impl DelegateComponent<FooComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@app.bar)>;
}
impl<__Context__, __Params__> IsProviderFor<FooComponent, __Context__, __Params__>
for App
where
    RedirectLookup<
        App,
        Path!(@app.bar),
    >: IsProviderFor<FooComponent, __Context__, __Params__>,
{}
fn main() {}
