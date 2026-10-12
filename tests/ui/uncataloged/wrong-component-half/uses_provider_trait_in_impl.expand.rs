#![feature(prelude_import)]
//! `#[uses(AreaCalculator)]` on a `#[cgp_impl]` provider, naming a provider trait where the
//! consumer trait belongs.
//!
//! The `E0107` "missing generics" keeps rustc's help `AreaCalculator<__Context__>`, which leaks the
//! generated name and is the wrong fix: the provider's body calls `self.area()`, a method of the
//! consumer trait `CanCalculateArea`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
impl<__Context__> CanCalculateArea for __Context__
where
    __Context__: AreaCalculator<__Context__>,
{
    fn area(&self) -> f64 {
        __Context__::area(self)
    }
}
pub trait AreaCalculator<
    __Context__,
>: IsProviderFor<AreaCalculatorComponent, __Context__, ()> {
    fn area(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> AreaCalculator<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<AreaCalculatorComponent>
        + IsProviderFor<AreaCalculatorComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        AreaCalculatorComponent,
    >>::Delegate: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent,
        >>::Delegate::area(__context__)
    }
}
pub struct AreaCalculatorComponent;
impl<__Context__> AreaCalculator<__Context__> for UseContext
where
    __Context__: CanCalculateArea,
{
    fn area(__context__: &__Context__) -> f64 {
        __Context__::area(__context__)
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for UseContext
where
    __Context__: CanCalculateArea,
{}
impl<__Context__, __Components__, __Path__> AreaCalculator<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::area(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<AreaCalculatorComponent, __Context__, ()>
        + AreaCalculator<__Context__>,
{}
pub trait CanDouble {
    fn double(&self) -> f64;
}
impl<__Context__> CanDouble for __Context__
where
    __Context__: Doubler<__Context__>,
{
    fn double(&self) -> f64 {
        __Context__::double(self)
    }
}
pub trait Doubler<__Context__>: IsProviderFor<DoublerComponent, __Context__, ()> {
    fn double(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> Doubler<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<DoublerComponent>
        + IsProviderFor<DoublerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        DoublerComponent,
    >>::Delegate: Doubler<__Context__>,
{
    fn double(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            DoublerComponent,
        >>::Delegate::double(__context__)
    }
}
pub struct DoublerComponent;
impl<__Context__> Doubler<__Context__> for UseContext
where
    __Context__: CanDouble,
{
    fn double(__context__: &__Context__) -> f64 {
        __Context__::double(__context__)
    }
}
impl<__Context__> IsProviderFor<DoublerComponent, __Context__, ()> for UseContext
where
    __Context__: CanDouble,
{}
impl<__Context__, __Components__, __Path__> Doubler<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Doubler<__Context__>,
{
    fn double(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::double(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<DoublerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<DoublerComponent, __Context__, ()>
        + Doubler<__Context__>,
{}
impl<__Context__> Doubler<__Context__> for DoubleArea
where
    __Context__: AreaCalculator,
{
    fn double(__context__: &__Context__) -> f64 {
        __context__.area() * 2.0
    }
}
impl<__Context__> IsProviderFor<DoublerComponent, __Context__, ()> for DoubleArea
where
    __Context__: AreaCalculator,
{}
pub struct DoubleArea;
fn main() {}
