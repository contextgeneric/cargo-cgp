#![feature(prelude_import)]
//! Edge case (expected behavior): a context derives `HasField` but has *no fields at
//! all*, while a getter names a field.
//!
//! `#[derive(HasField)]` emits one `HasField` impl per field, so on a struct with no
//! fields it emits *nothing*: byte-for-byte the same as a struct with no derive at all.
//! A fieldless derive leaves no trace in the generated program, so it is impossible to
//! tell whether the derive was even written; the two are the same program wherever
//! `HasField` is concerned. The tool therefore reports both fields the provider reads as
//! missing, without a `help`: it cannot tell whether the fix is a field or the derive. This
//! fixture pins that output.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md
//! (derive-missing variant).
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
///Component marker for [`CanCalculateArea`].
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
pub trait HasRectangleFields {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}
impl<__Context__> HasRectangleFields for __Context__
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
    __Context__: HasField<Symbol!("height"), Value = f64>,
{
    fn width(&self) -> f64 {
        ::core::clone::Clone::clone(
            HasField::get_field(self, ::core::marker::PhantomData::<Symbol!("width")>),
        )
    }
    fn height(&self) -> f64 {
        ::core::clone::Clone::clone(
            HasField::get_field(self, ::core::marker::PhantomData::<Symbol!("height")>),
        )
    }
}
impl<__Context__> AreaCalculator<__Context__> for RectangleArea
where
    __Context__: HasRectangleFields,
{
    fn area(__context__: &__Context__) -> f64 {
        __context__.width() * __context__.height()
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for RectangleArea
where
    __Context__: HasRectangleFields,
{}
///Provider `RectangleArea` for [`AreaCalculator`].
pub struct RectangleArea;
pub struct Rectangle {}
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = RectangleArea;
}
impl<
    __Context__,
    __Params__: ?Sized,
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
