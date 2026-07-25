#![feature(prelude_import)]
//! Usability: the wiring-note and header rewrites over a component that carries a generic
//! parameter, so `CanUseComponent`/`IsProviderFor` gain an extra type argument.
//!
//! `CanCalculateArea<Scalar>` is generic, so its provider trait is `AreaCalculator<Context,
//! Scalar>` and the check is `Rectangle: CanUseComponent<AreaCalculatorComponent, f64>`. The
//! provider still depends on a `width` field the `Rectangle` lacks, so the failure surfaces
//! through the same `IsProviderFor` / `CanUseComponent` chain as `base_area_1` — but now with
//! the extra `f64` parameter in the wiring traits. This fixture is the regression guard that
//! the driver's trait-renaming still names the traits when generic parameters are present.
//!
//! Exposes issues in docs/issues/usability.md. CGP error class:
//! https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanCalculateArea<Scalar> {
    fn area(&self) -> Scalar;
}
impl<__Context__, Scalar> CanCalculateArea<Scalar> for __Context__
where
    __Context__: AreaCalculator<__Context__, Scalar>,
{
    fn area(&self) -> Scalar {
        __Context__::area(self)
    }
}
pub trait AreaCalculator<
    __Context__,
    Scalar,
>: IsProviderFor<AreaCalculatorComponent, __Context__, (Scalar)> {
    fn area(__context__: &__Context__) -> Scalar;
}
impl<__Provider__, __Context__, Scalar> AreaCalculator<__Context__, Scalar>
for __Provider__
where
    __Provider__: DelegateComponent<AreaCalculatorComponent>
        + IsProviderFor<AreaCalculatorComponent, __Context__, (Scalar)>,
    <__Provider__ as DelegateComponent<
        AreaCalculatorComponent,
    >>::Delegate: AreaCalculator<__Context__, Scalar>,
{
    fn area(__context__: &__Context__) -> Scalar {
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent,
        >>::Delegate::area(__context__)
    }
}
pub struct AreaCalculatorComponent;
impl<__Context__, Scalar> AreaCalculator<__Context__, Scalar> for UseContext
where
    __Context__: CanCalculateArea<Scalar>,
{
    fn area(__context__: &__Context__) -> Scalar {
        __Context__::area(__context__)
    }
}
impl<__Context__, Scalar> IsProviderFor<AreaCalculatorComponent, __Context__, (Scalar)>
for UseContext
where
    __Context__: CanCalculateArea<Scalar>,
{}
impl<__Context__, Scalar, __Components__, __Path__> AreaCalculator<__Context__, Scalar>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Scalar)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Scalar)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Scalar)>>::Output,
    >>::Delegate: AreaCalculator<__Context__, Scalar>,
{
    fn area(__context__: &__Context__) -> Scalar {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Scalar)>>::Output,
        >>::Delegate::area(__context__)
    }
}
impl<
    __Context__,
    Scalar,
    __Components__,
    __Path__,
> IsProviderFor<AreaCalculatorComponent, __Context__, (Scalar)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Scalar)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Scalar)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Scalar)>>::Output,
    >>::Delegate: IsProviderFor<AreaCalculatorComponent, __Context__, (Scalar)>
        + AreaCalculator<__Context__, Scalar>,
{}
pub trait HasWidth {
    fn width(&self) -> f64;
}
impl<__Context__> HasWidth for __Context__
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
{
    fn width(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("width")>).clone()
    }
}
impl<__Context__> AreaCalculator<__Context__, f64> for RectangleArea
where
    __Context__: HasWidth,
{
    fn area(__context__: &__Context__) -> f64 {
        __context__.width() * __context__.width()
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, (f64)>
for RectangleArea
where
    __Context__: HasWidth,
{}
pub struct RectangleArea;
pub struct Rectangle {
    pub height: f64,
}
impl HasField<Symbol!("height")> for Rectangle {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("height")>,
    ) -> &Self::Value {
        &self.height
    }
}
impl HasFieldMut<Symbol!("height")> for Rectangle {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("height")>,
    ) -> &mut Self::Value {
        &mut self.height
    }
}
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
impl __CheckRectangle<AreaCalculatorComponent, f64> for Rectangle {}
fn main() {}
