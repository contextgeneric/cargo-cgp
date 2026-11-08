#![feature(prelude_import)]
//! A forwarding entry for a component with a type parameter, into a table that lacks the key.
//!
//! `ShowerComponent -> Bundle` forwards `CanShow<T>`'s key to a table that does not wire it. The
//! entry names no value for `T`, so no consumer obligation stands for it, and the forwarding-entry
//! anchor declines rather than check `CanShow<()>`, which nothing in the program asks for. The
//! compiler's two errors on the entry stay as rustc wrote them, including its suggestion to enable
//! `trivial_bounds`, which does not fix the program.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanShow<T> {
    fn show(&self, value: T);
}
impl<__Context__, T> CanShow<T> for __Context__
where
    __Context__: Shower<__Context__, T>,
{
    fn show(&self, value: T) {
        __Context__::show(self, value)
    }
}
pub trait Shower<__Context__, T>: IsProviderFor<ShowerComponent, __Context__, (T)> {
    fn show(__context__: &__Context__, value: T);
}
impl<__Provider__, __Context__, T> Shower<__Context__, T> for __Provider__
where
    __Provider__: DelegateComponent<ShowerComponent>
        + IsProviderFor<ShowerComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<
        ShowerComponent,
    >>::Delegate: Shower<__Context__, T>,
{
    fn show(__context__: &__Context__, value: T) {
        <__Provider__ as DelegateComponent<
            ShowerComponent,
        >>::Delegate::show(__context__, value)
    }
}
///Component marker for [`CanShow`].
pub struct ShowerComponent;
impl<__Context__, T> Shower<__Context__, T> for UseContext
where
    __Context__: CanShow<T>,
{
    fn show(__context__: &__Context__, value: T) {
        __Context__::show(__context__, value)
    }
}
impl<__Context__, T> IsProviderFor<ShowerComponent, __Context__, (T)> for UseContext
where
    __Context__: CanShow<T>,
{}
impl<__Context__, T, __Components__, __Path__> Shower<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: Shower<__Context__, T>,
{
    fn show(__context__: &__Context__, value: T) {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::show(__context__, value)
    }
}
impl<
    __Context__,
    T,
    __Components__,
    __Path__,
> IsProviderFor<ShowerComponent, __Context__, (T)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: IsProviderFor<ShowerComponent, __Context__, (T)>
        + Shower<__Context__, T>,
{}
pub trait CanFarewell {
    fn farewell(&self);
}
impl<__Context__> CanFarewell for __Context__
where
    __Context__: Farewell<__Context__>,
{
    fn farewell(&self) {
        __Context__::farewell(self)
    }
}
pub trait Farewell<__Context__>: IsProviderFor<FarewellComponent, __Context__, ()> {
    fn farewell(__context__: &__Context__);
}
impl<__Provider__, __Context__> Farewell<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<FarewellComponent>
        + IsProviderFor<FarewellComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        FarewellComponent,
    >>::Delegate: Farewell<__Context__>,
{
    fn farewell(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            FarewellComponent,
        >>::Delegate::farewell(__context__)
    }
}
///Component marker for [`CanFarewell`].
pub struct FarewellComponent;
impl<__Context__> Farewell<__Context__> for UseContext
where
    __Context__: CanFarewell,
{
    fn farewell(__context__: &__Context__) {
        __Context__::farewell(__context__)
    }
}
impl<__Context__> IsProviderFor<FarewellComponent, __Context__, ()> for UseContext
where
    __Context__: CanFarewell,
{}
impl<__Context__, __Components__, __Path__> Farewell<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Farewell<__Context__>,
{
    fn farewell(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::farewell(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<FarewellComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<FarewellComponent, __Context__, ()>
        + Farewell<__Context__>,
{}
impl<__Context__> Farewell<__Context__> for SayBye {
    fn farewell(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<FarewellComponent, __Context__, ()> for SayBye {}
///Provider `SayBye` for [`Farewell`].
pub struct SayBye;
///Wiring table `Bundle`.
pub struct Bundle;
impl DelegateComponent<FarewellComponent> for Bundle {
    type Delegate = SayBye;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<FarewellComponent, __Context__, __Params__> for Bundle
where
    SayBye: IsProviderFor<FarewellComponent, __Context__, __Params__>,
{}
pub struct Person;
impl DelegateComponent<ShowerComponent> for Person
where
    Bundle: DelegateComponent<ShowerComponent>,
{
    type Delegate = <Bundle as DelegateComponent<ShowerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ShowerComponent, __Context__, __Params__> for Person
where
    Bundle: DelegateComponent<ShowerComponent>,
    <Bundle as DelegateComponent<
        ShowerComponent,
    >>::Delegate: IsProviderFor<ShowerComponent, __Context__, __Params__>,
{}
fn main() {}
