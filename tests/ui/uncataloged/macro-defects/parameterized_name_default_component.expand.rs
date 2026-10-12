#![feature(prelude_import)]
//! A component named with a parameter and a `#[cgp_impl]` that relies on the default component name.
//!
//! The `E0107` carries a garbage suggestion (`#[cgp_impl(new SquareArea)]<Shape>`); the
//! `CGP-E032`/`CGP-E033` rewrites correctly stay off.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
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
>: IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)> {
    fn area(__context__: &__Context__, shape: &Shape) -> f64;
}
impl<__Provider__, __Context__, Shape> AreaCalculator<__Context__, Shape>
for __Provider__
where
    __Provider__: DelegateComponent<AreaCalculatorComponent<Shape>>
        + IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)>,
    <__Provider__ as DelegateComponent<
        AreaCalculatorComponent<Shape>,
    >>::Delegate: AreaCalculator<__Context__, Shape>,
{
    fn area(__context__: &__Context__, shape: &Shape) -> f64 {
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent<Shape>,
        >>::Delegate::area(__context__, shape)
    }
}
pub struct AreaCalculatorComponent<Shape>(pub ::core::marker::PhantomData<Shape>);
impl<__Context__, Shape> AreaCalculator<__Context__, Shape> for UseContext
where
    __Context__: CanCalculateArea<Shape>,
{
    fn area(__context__: &__Context__, shape: &Shape) -> f64 {
        __Context__::area(__context__, shape)
    }
}
impl<
    __Context__,
    Shape,
> IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)> for UseContext
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
> IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Shape)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Shape)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
    >>::Delegate: IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)>
        + AreaCalculator<__Context__, Shape>,
{}
pub struct Square(pub f64);
impl<__Context__> AreaCalculator<__Context__, Square> for SquareArea {
    fn area(__context__: &__Context__, shape: &Square) -> f64 {
        shape.0 * shape.0
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, (Square)>
for SquareArea {}
pub struct SquareArea;
fn main() {}
