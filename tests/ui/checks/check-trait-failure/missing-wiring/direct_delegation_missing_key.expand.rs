#![feature(prelude_import)]
//! A `->` delegation to a table that lacks the key.
//!
//! `GreeterComponent -> Bundle` where `Bundle` has no `GreeterComponent` entry. The check's
//! `[CGP-E001]` block ends at `[CGP-E110]` "provider `Bundle` does not contain any delegate entry
//! for `GreeterComponent`": `Person` has the entry, and the walk follows its forwarding bound to the
//! table that lacks the key. Two raw `E0277`s still precede it, spanning the whole block with a wrong
//! `trivial_bounds` suggestion, because the `->` entry's impls are not re-spanned onto the entry.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
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
pub struct SayBye;
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
