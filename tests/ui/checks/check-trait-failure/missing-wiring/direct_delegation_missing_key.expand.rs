#![feature(prelude_import)]
//! A `->` delegation to a table that lacks the key.
//!
//! `GreeterComponent -> Bundle` where `Bundle` has no `GreeterComponent` entry. The compiler first
//! reports the entry's own forwarding bound, `Bundle: DelegateComponent<GreeterComponent>`, and
//! the unsized `Delegate` projection it leaves, at the `-> Bundle` the bound is spanned onto. The
//! forwarding-entry anchor seeds `Person: CanGreet`, the consumer the entry wires, so the first of
//! them reads ``[CGP-E001] missing entry for `GreeterComponent` in provider `Bundle` `` over the
//! check's chain, and the second and the check's own report drop as the same failure.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
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
///Component marker for [`CanGreet`].
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
impl DelegateComponent<GreeterComponent> for Person
where
    Bundle: DelegateComponent<GreeterComponent>,
{
    type Delegate = <Bundle as DelegateComponent<GreeterComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    Bundle: DelegateComponent<GreeterComponent>,
    <Bundle as DelegateComponent<
        GreeterComponent,
    >>::Delegate: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckPerson<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckPerson<GreeterComponent, ()> for Person {}
fn main() {}
