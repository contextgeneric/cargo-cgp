#![feature(prelude_import)]
//! Acceptable failure: a missing wiring surfaced at the *use site* rather than at a
//! check. `App` wires `FooProviderComponent: DoFooWithBar` but never wires
//! `BarProviderComponent`, which `DoFooWithBar` depends on. CGP wiring is lazy, so the
//! gap does not surface at the `delegate_components!` block; it surfaces here, when
//! `foo()` is called, as an `E0599` "method exists but its trait bounds were not
//! satisfied".
//!
//! This is the missing-wiring analog of acceptable/use-site/missing_dependency.rs. With
//! no check impl to anchor on, the resolver recovers the context `App` from the
//! diagnostic's spans and re-checks the component it *does* wire
//! (`FooProviderComponent`), walking down to the unwired `BarProviderComponent`. The
//! result is the same `[CGP-E001]` header (the code kept `E0599`) over a
//! `root cause: context \`App\` does not contain any delegate entry for \`BarProviderComponent\`` note, with
//! rustc's misleading "use associated function syntax instead" advice dropped.
//!
//! See docs/errors/hidden/unsatisfied-dependency.md and
//! docs/implementation/typed-root-cause-resolution.md (the use-site path).
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
pub trait CanUseBar {
    fn bar(&self);
}
impl<__Context__> CanUseBar for __Context__
where
    __Context__: BarProvider<__Context__>,
{
    fn bar(&self) {
        __Context__::bar(self)
    }
}
pub trait BarProvider<
    __Context__,
>: IsProviderFor<BarProviderComponent, __Context__, ()> {
    fn bar(__context__: &__Context__);
}
impl<__Provider__, __Context__> BarProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<BarProviderComponent>
        + IsProviderFor<BarProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        BarProviderComponent,
    >>::Delegate: BarProvider<__Context__>,
{
    fn bar(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            BarProviderComponent,
        >>::Delegate::bar(__context__)
    }
}
pub struct BarProviderComponent;
impl<__Context__> BarProvider<__Context__> for UseContext
where
    __Context__: CanUseBar,
{
    fn bar(__context__: &__Context__) {
        __Context__::bar(__context__)
    }
}
impl<__Context__> IsProviderFor<BarProviderComponent, __Context__, ()> for UseContext
where
    __Context__: CanUseBar,
{}
impl<__Context__, __Components__, __Path__> BarProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: BarProvider<__Context__>,
{
    fn bar(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::bar(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<BarProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<BarProviderComponent, __Context__, ()>
        + BarProvider<__Context__>,
{}
impl<__Context__> FooProvider<__Context__> for DoFooWithBar
where
    __Context__: CanUseBar,
{
    fn foo(__context__: &__Context__) {
        __context__.bar()
    }
}
impl<__Context__> IsProviderFor<FooProviderComponent, __Context__, ()> for DoFooWithBar
where
    __Context__: CanUseBar,
{}
pub struct DoFooWithBar;
pub struct App;
impl DelegateComponent<FooProviderComponent> for App {
    type Delegate = DoFooWithBar;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<FooProviderComponent, __Context__, __Params__> for App
where
    DoFooWithBar: IsProviderFor<FooProviderComponent, __Context__, __Params__>,
{}
fn main() {
    App.foo();
}
