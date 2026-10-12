#![feature(prelude_import)]
//! `open` plus a `UseDelegate` table for the same component.
//!
//! The opened route looks `AreaCalculatorComponent` up with its parameter `Shape` appended, so the
//! `[CGP-E013]` help names the generic key `@AreaCalculatorComponent.<Shape> Shape` to wire the
//! direct entry's `UseDelegate<AreaTable>` under, rather than the bare path the lookup never reads.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
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
pub struct Rectangle;
impl<__Context__> AreaCalculator<__Context__, Rectangle> for RectangleArea {
    fn area(__context__: &__Context__, _shape: &Rectangle) -> f64 {
        1.0
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, (Rectangle)>
for RectangleArea {}
pub struct RectangleArea;
pub struct App;
impl DelegateComponent<AreaCalculatorComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@AreaCalculatorComponent)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@AreaCalculatorComponent),
    >: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<AreaCalculatorComponent, PathCons<Rectangle, __Wildcard__>>>
for App {
    type Delegate = RectangleArea;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<AreaCalculatorComponent, PathCons<Rectangle, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    RectangleArea: IsProviderFor<
        PathCons<AreaCalculatorComponent, PathCons<Rectangle, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
pub struct AreaTable;
impl DelegateComponent<AreaCalculatorComponent> for App {
    type Delegate = UseDelegate<AreaTable>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for App
where
    UseDelegate<
        AreaTable,
    >: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
impl DelegateComponent<Rectangle> for AreaTable {
    type Delegate = RectangleArea;
}
impl<__Context__, __Params__: ?Sized> IsProviderFor<Rectangle, __Context__, __Params__>
for AreaTable
where
    RectangleArea: IsProviderFor<Rectangle, __Context__, __Params__>,
{}
fn main() {}
