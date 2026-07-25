#![feature(prelude_import)]
//! Acceptable failure: a *transitive* missing wiring. `DoFooWithBar` carries the
//! impl-side dependency `#[uses(CanUseBar)]`, so `App` can only use `FooProvider`
//! if it also wires the `BarProvider` component — but `App` wires only
//! `FooProviderComponent`, never `BarProviderComponent`. The check therefore fails
//! not because a field is missing but because a component the wired provider needs
//! is not delegated at all.
//!
//! This is the missing-wiring analog of acceptable/fields/missing_dependency.rs: the
//! typed resolver walks the same `CanUseComponent` → `IsProviderFor` chain, but the
//! terminal leaf is an unmet `DelegateComponent<BarProviderComponent>` on the context
//! rather than an unmet `HasField`. It renders as a `[CGP-E001]` header over one
//! `root cause: context \`App\` does not contain any delegate entry for \`BarProviderComponent\`` note, with the
//! dependency chain bottoming out at the `CanUseBar` capability the missing component
//! would supply.
//!
//! See cgp-knowledge-base/cgp/errors/checks/check-trait-failure.md (its "the wiring
//! is missing" face) and
//! cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md.
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
impl<__Context__> BarProvider<__Context__> for DoBar {
    fn bar(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<BarProviderComponent, __Context__, ()> for DoBar {}
pub struct DoBar;
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
trait __CanUseApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CanUseApp<FooProviderComponent, ()> for App {}
fn main() {}
