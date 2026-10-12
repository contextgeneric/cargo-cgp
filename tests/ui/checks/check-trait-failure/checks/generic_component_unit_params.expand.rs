#![feature(prelude_import)]
//! `delegate_and_check_components!` wiring a component with a type parameter, without
//! `#[check_params(...)]`.
//!
//! The derived check tests `CanCalculateArea` at the unit parameters `()`, which `RectangleArea`,
//! written for `Rectangle`, does not provide. The tool reports `[CGP-E060]` with
//! `#[check_params(Rectangle)]` in the help, where rustc reports the provider as not implementing
//! the provider trait and only its help "expected `Rectangle`, found `()`" hints at the cause.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
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
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}
impl<__Context__> AreaCalculator<__Context__, Rectangle> for RectangleArea {
    fn area(__context__: &__Context__, shape: &Rectangle) -> f64 {
        shape.width * shape.height
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, (Rectangle)>
for RectangleArea {}
pub struct RectangleArea;
pub struct App;
impl DelegateComponent<AreaCalculatorComponent> for App {
    type Delegate = RectangleArea;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for App
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
trait __CanUseApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CanUseApp<AreaCalculatorComponent, ()> for App {}
fn main() {}
