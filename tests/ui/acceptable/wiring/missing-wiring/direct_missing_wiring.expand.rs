#![feature(prelude_import)]
//! Acceptable failure: the simplest missing wiring — a `check_components!` asserts a
//! component the context does not wire at all. There is no transitive dependency here;
//! `App` has an empty wiring table, so `App: DelegateComponent<FooProviderComponent>`
//! is unmet directly under the checked `CanUseComponent` obligation.
//!
//! The dependency chain is therefore a single node — the `CanUseFoo` consumer the
//! missing wiring would provide — with the same `[CGP-E001]` header and a
//! `root cause: context \`App\` does not contain any delegate entry for \`FooProviderComponent\`` note. It pins
//! that the resolver reports a bare unwired component, not only one reached through a
//! provider's impl-side dependency (that transitive case is basic_missing_wiring.rs).
//!
//! See docs/implementation/typed-root-cause-resolution.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanUseFoo {
    fn foo(&self);
}
impl<__Context__> CanUseFoo for __Context__
where
    __Context__: FooProvider<__Context__>,
{
    fn foo(&self) {
        __Context__::foo(self)
    }
}
pub trait FooProvider<
    __Context__,
>: IsProviderFor<FooProviderComponent, __Context__, ()> {
    fn foo(__context__: &__Context__);
}
impl<__Provider__, __Context__> FooProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<FooProviderComponent>
        + IsProviderFor<FooProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        FooProviderComponent,
    >>::Delegate: FooProvider<__Context__>,
{
    fn foo(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            FooProviderComponent,
        >>::Delegate::foo(__context__)
    }
}
pub struct FooProviderComponent;
impl<__Context__> FooProvider<__Context__> for UseContext
where
    __Context__: CanUseFoo,
{
    fn foo(__context__: &__Context__) {
        __Context__::foo(__context__)
    }
}
impl<__Context__> IsProviderFor<FooProviderComponent, __Context__, ()> for UseContext
where
    __Context__: CanUseFoo,
{}
impl<__Context__, __Components__, __Path__> FooProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: FooProvider<__Context__>,
{
    fn foo(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::foo(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<FooProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<FooProviderComponent, __Context__, ()>
        + FooProvider<__Context__>,
{}
impl<__Context__> FooProvider<__Context__> for DoFoo {
    fn foo(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<FooProviderComponent, __Context__, ()> for DoFoo {}
pub struct DoFoo;
pub struct App;
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<FooProviderComponent, ()> for App {}
fn main() {}
