#![feature(prelude_import)]
//! Acceptable failure: a wiring gap surfaced inside a *hand-written* `impl Trait for Context`
//! block — where the caret sits on the impl, never on the context's own type definition. The
//! impl-site anchor recovers the context from the enclosing impl's `Self` type and the *exact*
//! failing obligation (with its concrete component parameter, `CanCalculateArea<Rectangle>`) from
//! the impl's CGP consumer supertrait, then walks it as a check entry would — but heads the tree
//! with the impl's *own* trait, the wrapper the programmer wrote, so the failure reads at their
//! code and descends from there.
//!
//! This is the shape the `cgp-examples/transfer` walkthrough hits with its per-endpoint
//! `impl CanHandleApiSend<Api> for MockApp` blocks: a wrapper trait carrying a generic CGP
//! consumer trait as a supertrait, implemented directly on the context to add a bound (there,
//! `Send`) the component cannot express. Here `App` wires `AreaCalculatorComponent` for
//! `Rectangle` through `ScaleArea`, which depends on the unwired `scale_factor` field; the
//! unsatisfied `App: CanCalculateArea<Rectangle>` supertrait then fails *inside* the
//! `impl CanCalculateAreaChecked<Rectangle> for App` block, at the impl header (`E0277`) and the
//! forwarding `self.area(..)` call (`E0599`). Both recover the same cause and collapse to one
//! block, headed `[CGP-E009] the trait \`CanCalculateAreaChecked<Rectangle>\`` — a wrapper is a
//! plain trait, not a CGP consumer, so it reads "the trait", not "the consumer trait" — over a
//! tree that leads with `CanCalculateAreaChecked`, then its `CanCalculateArea` supertrait, down to
//! the missing field.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md (the impl-site
//! path).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Rectangle;
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: PhantomData<Shape>) -> f64;
}
impl<__Context__, Shape> CanCalculateArea<Shape> for __Context__
where
    __Context__: AreaCalculator<__Context__, Shape>,
{
    fn area(&self, shape: PhantomData<Shape>) -> f64 {
        __Context__::area(self, shape)
    }
}
pub trait AreaCalculator<
    __Context__,
    Shape,
>: IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)> {
    fn area(__context__: &__Context__, shape: PhantomData<Shape>) -> f64;
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
    fn area(__context__: &__Context__, shape: PhantomData<Shape>) -> f64 {
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
    fn area(__context__: &__Context__, shape: PhantomData<Shape>) -> f64 {
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
    fn area(__context__: &__Context__, shape: PhantomData<Shape>) -> f64 {
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
pub trait HasScaleFactor {
    fn scale_factor(&self) -> &f64;
}
impl<__Context__> HasScaleFactor for __Context__
where
    __Context__: HasField<Symbol!("scale_factor"), Value = f64>,
{
    fn scale_factor(&self) -> &f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("scale_factor")>)
    }
}
impl<__Context__, Shape> AreaCalculator<__Context__, Shape> for ScaleArea
where
    __Context__: HasScaleFactor,
{
    fn area(__context__: &__Context__, _shape: PhantomData<Shape>) -> f64 {
        *__context__.scale_factor()
    }
}
impl<__Context__, Shape> IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)>
for ScaleArea
where
    __Context__: HasScaleFactor,
{}
pub struct ScaleArea;
pub struct App;
impl DelegateComponent<AreaCalculatorComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@AreaCalculatorComponent)>;
}
impl<
    __Context__,
    __Params__,
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
    type Delegate = ScaleArea;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<AreaCalculatorComponent, PathCons<Rectangle, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    ScaleArea: IsProviderFor<
        PathCons<AreaCalculatorComponent, PathCons<Rectangle, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
pub trait CanCalculateAreaChecked<Shape>: CanCalculateArea<Shape> {
    fn area_checked(&self, shape: PhantomData<Shape>) -> f64;
}
impl CanCalculateAreaChecked<Rectangle> for App {
    fn area_checked(&self, shape: PhantomData<Rectangle>) -> f64 {
        self.area(shape)
    }
}
fn main() {}
