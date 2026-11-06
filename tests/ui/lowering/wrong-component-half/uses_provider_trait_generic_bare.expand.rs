#![feature(prelude_import)]
//! `#[uses(AreaCalculator)]` on a `#[cgp_impl]` provider, naming a generic component's provider
//! trait with no arguments where its consumer trait belongs.
//!
//! rustc reports `E0107`, missing generics for trait `AreaCalculator`, as for a component without
//! parameters, and the tool reshapes it into `[CGP-E037]`. The program wrote no argument for the
//! component's `Shape`, so the help names the consumer trait bare, `#[uses(CanCalculateArea)]`,
//! which still needs the argument the program means.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}
impl<__Context__, Shape> CanCalculateArea<Shape> for __Context__
where
    __Context__: AreaCalculator<__Context__, Shape>,
{
    fn area(&self, shape: &Shape) -> f64 {
        __Context__::area(self, shape)
    }
}
pub trait AreaCalculator<
    __Context__,
    Shape,
>: IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)> {
    fn area(__context__: &__Context__, shape: &Shape) -> f64;
}
impl<__Provider__, __Context__, Shape> AreaCalculator<__Context__, Shape>
for __Provider__
where
    __Provider__: DelegateComponent<AreaCalculatorComponent>
        + IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)>,
    <__Provider__ as DelegateComponent<
        AreaCalculatorComponent,
    >>::Delegate: AreaCalculator<__Context__, Shape>,
{
    fn area(__context__: &__Context__, shape: &Shape) -> f64 {
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent,
        >>::Delegate::area(__context__, shape)
    }
}
pub struct AreaCalculatorComponent;
impl<__Context__, Shape> AreaCalculator<__Context__, Shape> for UseContext
where
    __Context__: CanCalculateArea<Shape>,
{
    fn area(__context__: &__Context__, shape: &Shape) -> f64 {
        __Context__::area(__context__, shape)
    }
}
impl<__Context__, Shape> IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)>
for UseContext
where
    __Context__: CanCalculateArea<Shape>,
{}
impl<__Context__, Shape, __Components__, __Path__> AreaCalculator<__Context__, Shape>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Shape)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Shape)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
    >>::Delegate: AreaCalculator<__Context__, Shape>,
{
    fn area(__context__: &__Context__, shape: &Shape) -> f64 {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
        >>::Delegate::area(__context__, shape)
    }
}
impl<
    __Context__,
    Shape,
    __Components__,
    __Path__,
> IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Shape)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Shape)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
    >>::Delegate: IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)>
        + AreaCalculator<__Context__, Shape>,
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
pub struct Rectangle;
impl<__Context__> Doubler<__Context__> for DoubleArea
where
    __Context__: AreaCalculator,
{
    fn double(__context__: &__Context__) -> f64 {
        __context__.area(&Rectangle) * 2.0
    }
}
impl<__Context__> IsProviderFor<DoublerComponent, __Context__, ()> for DoubleArea
where
    __Context__: AreaCalculator,
{}
pub struct DoubleArea;
fn main() {}
