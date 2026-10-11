#![feature(prelude_import)]
//! A two-component wiring cycle checked at both components, so the second check re-walks a node
//! the first walk cut. `DoA` needs `CanB` and a `width` field the context lacks; `DoB` needs `CanA`
//! back. The walk from `CanA` reaches `CanB`, whose subtree loops back to `CanA` and is cut there,
//! so it is incomplete and must not be cached: the walk from `CanB` then resolves it afresh and
//! finds the missing field through `CanA`. Caching the cut subtree would leave that second walk an
//! empty result and its error unrewritten.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`); the cache rule is in
//! `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanA {
    fn a(&self);
}
impl<__Context__> CanA for __Context__
where
    __Context__: ProviderA<__Context__>,
{
    fn a(&self) {
        __Context__::a(self)
    }
}
pub trait ProviderA<__Context__>: IsProviderFor<ProviderAComponent, __Context__, ()> {
    fn a(__context__: &__Context__);
}
impl<__Provider__, __Context__> ProviderA<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ProviderAComponent>
        + IsProviderFor<ProviderAComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ProviderAComponent,
    >>::Delegate: ProviderA<__Context__>,
{
    fn a(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<ProviderAComponent>>::Delegate::a(__context__)
    }
}
pub struct ProviderAComponent;
impl<__Context__> ProviderA<__Context__> for UseContext
where
    __Context__: CanA,
{
    fn a(__context__: &__Context__) {
        __Context__::a(__context__)
    }
}
impl<__Context__> IsProviderFor<ProviderAComponent, __Context__, ()> for UseContext
where
    __Context__: CanA,
{}
impl<__Context__, __Components__, __Path__> ProviderA<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: ProviderA<__Context__>,
{
    fn a(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::a(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ProviderAComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ProviderAComponent, __Context__, ()>
        + ProviderA<__Context__>,
{}
pub trait CanB {
    fn b(&self);
}
impl<__Context__> CanB for __Context__
where
    __Context__: ProviderB<__Context__>,
{
    fn b(&self) {
        __Context__::b(self)
    }
}
pub trait ProviderB<__Context__>: IsProviderFor<ProviderBComponent, __Context__, ()> {
    fn b(__context__: &__Context__);
}
impl<__Provider__, __Context__> ProviderB<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ProviderBComponent>
        + IsProviderFor<ProviderBComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ProviderBComponent,
    >>::Delegate: ProviderB<__Context__>,
{
    fn b(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<ProviderBComponent>>::Delegate::b(__context__)
    }
}
pub struct ProviderBComponent;
impl<__Context__> ProviderB<__Context__> for UseContext
where
    __Context__: CanB,
{
    fn b(__context__: &__Context__) {
        __Context__::b(__context__)
    }
}
impl<__Context__> IsProviderFor<ProviderBComponent, __Context__, ()> for UseContext
where
    __Context__: CanB,
{}
impl<__Context__, __Components__, __Path__> ProviderB<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: ProviderB<__Context__>,
{
    fn b(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::b(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ProviderBComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ProviderBComponent, __Context__, ()>
        + ProviderB<__Context__>,
{}
pub trait HasWidth {
    fn width(&self) -> f64;
}
impl<__Context__> HasWidth for __Context__
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
{
    fn width(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("width")>).clone()
    }
}
impl<__Context__> ProviderA<__Context__> for DoA
where
    __Context__: CanB + HasWidth,
{
    fn a(__context__: &__Context__) {
        let _ = __context__.width();
        __context__.b();
    }
}
impl<__Context__> IsProviderFor<ProviderAComponent, __Context__, ()> for DoA
where
    __Context__: CanB + HasWidth,
{}
pub struct DoA;
impl<__Context__> ProviderB<__Context__> for DoB
where
    __Context__: CanA,
{
    fn b(__context__: &__Context__) {
        __context__.a();
    }
}
impl<__Context__> IsProviderFor<ProviderBComponent, __Context__, ()> for DoB
where
    __Context__: CanA,
{}
pub struct DoB;
pub struct App {}
impl DelegateComponent<ProviderAComponent> for App {
    type Delegate = DoA;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ProviderAComponent, __Context__, __Params__> for App
where
    DoA: IsProviderFor<ProviderAComponent, __Context__, __Params__>,
{}
impl DelegateComponent<ProviderBComponent> for App {
    type Delegate = DoB;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ProviderBComponent, __Context__, __Params__> for App
where
    DoB: IsProviderFor<ProviderBComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ProviderAComponent, ()> for App {}
impl __CheckApp<ProviderBComponent, ()> for App {}
fn main() {}
