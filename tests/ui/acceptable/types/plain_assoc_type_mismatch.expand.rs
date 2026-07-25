#![feature(prelude_import)]
//! The negative counterpart of [`abstract_type_mismatch`](abstract_type_mismatch.rs): the same
//! projection failure on a trait that is **not** a CGP abstract-type component. `HasUnit` is a
//! plain Rust trait with an associated type, implemented directly on the context — no
//! `#[cgp_type]`, so no provider trait, no component marker, and no `UseType` blanket.
//! `RectangleArea` pins its `Unit` to `f64` through an ordinary `where` clause (the form
//! [`#[use_type]`](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/reference/attributes/use_type.md)
//! deliberately does not cover, since the trait is not an abstract-type component), while
//! `Rectangle` supplies `u32`.
//!
//! The failure is reshaped into the same `[CGP-E017]` class — the mechanism is the projection, not
//! the trait — but the wording turns on the trait's fingerprint, and that is what this fixture pins.
//! Because `HasUnit`'s provider trait carries no `UseType` impl, it reads `associated type` rather
//! than `abstract type` and carries **no** `help`: there is no wiring entry to change, so suggesting
//! `UseType<f64>` here would name a fix that does not exist. The concrete type is fixed by the
//! `impl HasUnit for Rectangle` block, and that is where a reader must go.
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
/// A plain associated-type trait — deliberately *not* `#[cgp_type]`, so nothing about it is wired.
pub trait HasUnit {
    type Unit;
}
pub struct Rectangle;
impl HasUnit for Rectangle {
    type Unit = u32;
}
impl<__Context__> AreaCalculator<__Context__> for RectangleArea
where
    __Context__: HasUnit<Unit = f64>,
{
    fn area(__context__: &__Context__) -> f64 {
        1.0
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for RectangleArea
where
    __Context__: HasUnit<Unit = f64>,
{}
pub struct RectangleArea;
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = RectangleArea;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
trait __CheckRectangle<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}
fn main() {}
