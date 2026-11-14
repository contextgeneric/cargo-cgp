#![feature(prelude_import)]
//! A provider header for a component whose key takes a parameter, written without the trait's own
//! argument.
//!
//! `impl AreaCalculator` omits the `Shape` argument the provider trait takes, so rustc reports an
//! `E0107` "missing generics for trait `AreaCalculator`" on the trait's name, beside the one the
//! derived key `AreaCalculatorComponent` gets at the same name. Only the key's is `[CGP-E070]`,
//! whose attribute form keeps the key's parameter `Shape`, since the header writes no argument for
//! it; the trait's own arity mistake keeps rustc's error.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
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
///Provider trait for [`CanCalculateArea`]. A provider imports this trait's component key, [`AreaCalculatorComponent`], beside it.
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
///Component marker for [`CanCalculateArea`].
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
impl<__Context__> AreaCalculator<__Context__> for AnyArea {
    fn area(__context__: &__Context__, _shape: &u32) -> f64 {
        1.0
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for AnyArea {}
///Provider `AnyArea` for [`AreaCalculator`].
pub struct AnyArea;
fn main() {}
