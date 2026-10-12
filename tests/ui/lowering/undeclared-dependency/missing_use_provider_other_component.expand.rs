#![feature(prelude_import)]
//! A higher-order provider calling an inner provider of another component without importing it.
//!
//! `ScaledArea<InnerShape>` implements `AreaCalculator` but calls `InnerShape::perimeter(self)`, a
//! method of the `PerimeterCalculator` provider trait, without
//! `#[use_provider(InnerShape: PerimeterCalculator)]`. The enclosing impl's own trait does not
//! declare `perimeter`, so the provider trait to import is found among every provider trait that
//! declares the method. rustc reports `E0599` on the unbounded type parameter.
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
pub trait CanCalculatePerimeter {
    fn perimeter(&self) -> f64;
}
impl<__Context__> CanCalculatePerimeter for __Context__
where
    __Context__: PerimeterCalculator<__Context__>,
{
    fn perimeter(&self) -> f64 {
        __Context__::perimeter(self)
    }
}
pub trait PerimeterCalculator<
    __Context__,
>: IsProviderFor<PerimeterCalculatorComponent, __Context__, ()> {
    fn perimeter(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> PerimeterCalculator<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<PerimeterCalculatorComponent>
        + IsProviderFor<PerimeterCalculatorComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        PerimeterCalculatorComponent,
    >>::Delegate: PerimeterCalculator<__Context__>,
{
    fn perimeter(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            PerimeterCalculatorComponent,
        >>::Delegate::perimeter(__context__)
    }
}
pub struct PerimeterCalculatorComponent;
impl<__Context__> PerimeterCalculator<__Context__> for UseContext
where
    __Context__: CanCalculatePerimeter,
{
    fn perimeter(__context__: &__Context__) -> f64 {
        __Context__::perimeter(__context__)
    }
}
impl<__Context__> IsProviderFor<PerimeterCalculatorComponent, __Context__, ()>
for UseContext
where
    __Context__: CanCalculatePerimeter,
{}
impl<__Context__, __Components__, __Path__> PerimeterCalculator<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: PerimeterCalculator<__Context__>,
{
    fn perimeter(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::perimeter(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<PerimeterCalculatorComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<PerimeterCalculatorComponent, __Context__, ()>
        + PerimeterCalculator<__Context__>,
{}
impl<__Context__, InnerShape> AreaCalculator<__Context__> for ScaledArea<InnerShape>
where
    __Context__: HasField<Symbol!("scale_factor"), Value = f64>,
{
    fn area(__context__: &__Context__) -> f64 {
        let scale_factor: f64 = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("scale_factor")>)
            .clone();
        InnerShape::perimeter(__context__) * scale_factor
    }
}
impl<__Context__, InnerShape> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for ScaledArea<InnerShape>
where
    __Context__: HasField<Symbol!("scale_factor"), Value = f64>,
{}
pub struct ScaledArea<InnerShape>(pub ::core::marker::PhantomData<InnerShape>);
fn main() {}
