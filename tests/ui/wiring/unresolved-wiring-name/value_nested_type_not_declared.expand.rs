#![feature(prelude_import)]
//! A type nothing declares, nested in a wiring value's generic arguments rather than in an
//! `@`-path.
//!
//! `AreaCalculatorComponent: UseDelegate<Missing>` names an undeclared `Missing` inside the value.
//! It is neither the whole value nor a segment of a path, so the tool leaves the compiler's
//! `E0425`s as rustc wrote them, with its suggestion to add a type parameter, which no wiring entry
//! can take.
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
impl<__Context__, Shape, __Components__, __Delegate__> AreaCalculator<__Context__, Shape>
for UseDelegate<__Components__>
where
    __Components__: DelegateComponent<(Shape), Delegate = __Delegate__>,
    __Delegate__: AreaCalculator<__Context__, Shape>,
{
    fn area(__context__: &__Context__, shape: &Shape) -> f64 {
        __Delegate__::area(__context__, shape)
    }
}
impl<
    __Context__,
    Shape,
    __Components__,
    __Delegate__,
> IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)>
for UseDelegate<__Components__>
where
    __Components__: DelegateComponent<(Shape), Delegate = __Delegate__>,
    __Delegate__: IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)>
        + AreaCalculator<__Context__, Shape>,
{}
pub struct App;
impl DelegateComponent<AreaCalculatorComponent> for App {
    type Delegate = UseDelegate<Missing>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for App
where
    UseDelegate<
        Missing,
    >: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
fn main() {}
