#![feature(prelude_import)]
//! Acceptable failure: two independent missing wirings surface as two separate root
//! causes. `DoFooWithBarBaz` depends on both `CanUseBar` and `CanUseBaz`
//! (`#[uses(CanUseBar, CanUseBaz)]`), and `App` wires neither `BarProviderComponent`
//! nor `BazProviderComponent`, so both are missing.
//!
//! This is the missing-wiring analog of acceptable/fields/parallel_branches.rs: the
//! resolver follows *every* unmet dependency, not just the first the next-generation
//! solver stops at, so a single `[CGP-E001]` header carries two `root cause: missing
//! wiring …` notes — one per unwired component — each with its own dependency chain.
//! A regression that followed only the first unmet bound would report one and hide the
//! other.
//!
//! See docs/implementation/typed-root-cause-resolution.md (parallel branches).
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
pub trait CanUseBaz {
    fn baz(&self);
}
impl<__Context__> CanUseBaz for __Context__
where
    __Context__: BazProvider<__Context__>,
{
    fn baz(&self) {
        __Context__::baz(self)
    }
}
pub trait BazProvider<
    __Context__,
>: IsProviderFor<BazProviderComponent, __Context__, ()> {
    fn baz(__context__: &__Context__);
}
impl<__Provider__, __Context__> BazProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<BazProviderComponent>
        + IsProviderFor<BazProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        BazProviderComponent,
    >>::Delegate: BazProvider<__Context__>,
{
    fn baz(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            BazProviderComponent,
        >>::Delegate::baz(__context__)
    }
}
pub struct BazProviderComponent;
impl<__Context__> BazProvider<__Context__> for UseContext
where
    __Context__: CanUseBaz,
{
    fn baz(__context__: &__Context__) {
        __Context__::baz(__context__)
    }
}
impl<__Context__> IsProviderFor<BazProviderComponent, __Context__, ()> for UseContext
where
    __Context__: CanUseBaz,
{}
impl<__Context__, __Components__, __Path__> BazProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: BazProvider<__Context__>,
{
    fn baz(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::baz(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<BazProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<BazProviderComponent, __Context__, ()>
        + BazProvider<__Context__>,
{}
impl<__Context__> FooProvider<__Context__> for DoFooWithBarBaz
where
    __Context__: CanUseBar + CanUseBaz,
{
    fn foo(__context__: &__Context__) {
        __context__.bar();
        __context__.baz();
    }
}
impl<__Context__> IsProviderFor<FooProviderComponent, __Context__, ()>
for DoFooWithBarBaz
where
    __Context__: CanUseBar + CanUseBaz,
{}
pub struct DoFooWithBarBaz;
pub struct App;
impl DelegateComponent<FooProviderComponent> for App {
    type Delegate = DoFooWithBarBaz;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<FooProviderComponent, __Context__, __Params__> for App
where
    DoFooWithBarBaz: IsProviderFor<FooProviderComponent, __Context__, __Params__>,
{}
trait __CanUseApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CanUseApp<FooProviderComponent, ()> for App {}
fn main() {}
