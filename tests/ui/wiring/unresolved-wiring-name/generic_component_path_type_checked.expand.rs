#![feature(prelude_import)]
//! An undeclared type in the path a component with a parameter is redirected to, and a check of the
//! component at a parameter.
//!
//! `AreaCalculatorComponent => @app.Missing` names a type nothing declares, so rustc reports an
//! `E0425` at each use of the path, and the check then overflows (`E0275`) on
//! `App: CanUseComponent<AreaCalculatorComponent, Rectangle>`, a requirement the redirect leaves
//! unprovable. The tool reports one `[CGP-E044]` at the redirect but keeps the overflow, though it
//! drops the same overflow for a component without parameters (`path_segment_not_declared`).
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
impl<__Context__, Shape> AreaCalculator<__Context__, Shape> for RectangleArea {
    fn area(__context__: &__Context__, _shape: &Shape) -> f64 {
        0.0
    }
}
impl<__Context__, Shape> IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)>
for RectangleArea {}
pub struct RectangleArea;
pub struct Rectangle;
pub struct App;
impl DelegateComponent<AreaCalculatorComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@app.Missing)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@app.Missing),
    >: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<Missing, __Wildcard__>>> for App {
    type Delegate = RectangleArea;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<Missing, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    RectangleArea: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<Missing, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<AreaCalculatorComponent, Rectangle> for App {}
fn main() {}
