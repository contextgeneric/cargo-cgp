#![feature(prelude_import)]
//! Two higher-order providers call an inner provider's `area` without `#[use_provider]`, where two
//! components' provider traits declare an `area`.
//!
//! `ScaledArea` implements `AreaCalculator`, which declares `area`, so the tool reports `[CGP-E031]`
//! naming that trait. `DescribeArea` implements `Describer`, which does not, so the tool cannot tell
//! which of the two traits is meant, and leaves rustc's `E0599`, whose suggestions name the generated
//! `__Context__`.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
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
pub trait CanCalculateFloorArea {
    fn area(&self) -> f64;
}
impl<__Context__> CanCalculateFloorArea for __Context__
where
    __Context__: FloorAreaCalculator<__Context__>,
{
    fn area(&self) -> f64 {
        __Context__::area(self)
    }
}
pub trait FloorAreaCalculator<
    __Context__,
>: IsProviderFor<FloorAreaCalculatorComponent, __Context__, ()> {
    fn area(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> FloorAreaCalculator<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<FloorAreaCalculatorComponent>
        + IsProviderFor<FloorAreaCalculatorComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        FloorAreaCalculatorComponent,
    >>::Delegate: FloorAreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            FloorAreaCalculatorComponent,
        >>::Delegate::area(__context__)
    }
}
pub struct FloorAreaCalculatorComponent;
impl<__Context__> FloorAreaCalculator<__Context__> for UseContext
where
    __Context__: CanCalculateFloorArea,
{
    fn area(__context__: &__Context__) -> f64 {
        __Context__::area(__context__)
    }
}
impl<__Context__> IsProviderFor<FloorAreaCalculatorComponent, __Context__, ()>
for UseContext
where
    __Context__: CanCalculateFloorArea,
{}
impl<__Context__, __Components__, __Path__> FloorAreaCalculator<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: FloorAreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::area(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<FloorAreaCalculatorComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<FloorAreaCalculatorComponent, __Context__, ()>
        + FloorAreaCalculator<__Context__>,
{}
pub trait CanDescribe {
    fn describe(&self) -> String;
}
impl<__Context__> CanDescribe for __Context__
where
    __Context__: Describer<__Context__>,
{
    fn describe(&self) -> String {
        __Context__::describe(self)
    }
}
pub trait Describer<__Context__>: IsProviderFor<DescriberComponent, __Context__, ()> {
    fn describe(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Describer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<DescriberComponent>
        + IsProviderFor<DescriberComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        DescriberComponent,
    >>::Delegate: Describer<__Context__>,
{
    fn describe(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            DescriberComponent,
        >>::Delegate::describe(__context__)
    }
}
pub struct DescriberComponent;
impl<__Context__> Describer<__Context__> for UseContext
where
    __Context__: CanDescribe,
{
    fn describe(__context__: &__Context__) -> String {
        __Context__::describe(__context__)
    }
}
impl<__Context__> IsProviderFor<DescriberComponent, __Context__, ()> for UseContext
where
    __Context__: CanDescribe,
{}
impl<__Context__, __Components__, __Path__> Describer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Describer<__Context__>,
{
    fn describe(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::describe(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<DescriberComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<DescriberComponent, __Context__, ()>
        + Describer<__Context__>,
{}
impl<__Context__, Inner> AreaCalculator<__Context__> for ScaledArea<Inner> {
    fn area(__context__: &__Context__) -> f64 {
        Inner::area(__context__) * 2.0
    }
}
impl<__Context__, Inner> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for ScaledArea<Inner> {}
pub struct ScaledArea<Inner>(pub ::core::marker::PhantomData<Inner>);
impl<__Context__, Inner> Describer<__Context__> for DescribeArea<Inner> {
    fn describe(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0}", Inner::area(__context__)))
        })
    }
}
impl<__Context__, Inner> IsProviderFor<DescriberComponent, __Context__, ()>
for DescribeArea<Inner> {}
pub struct DescribeArea<Inner>(pub ::core::marker::PhantomData<Inner>);
fn main() {}
