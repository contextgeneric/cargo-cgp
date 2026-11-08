#![feature(prelude_import)]
//! A `#[cgp_impl]` header naming the consumer trait, beside a field of the wrong type.
//!
//! Two mistakes: `impl CanCalculateArea` should be `impl AreaCalculator`, and `Rectangle`'s `height`
//! is an `i32` where the provider reads an `f64`. The compiler reports the field type as an `E0271`
//! on the check, through the provider's requirements. The tool reports the header as `[CGP-E032]`
//! and reads the requirements off the impl as written, so the field type is `[CGP-E004]`, the block
//! the program gives once the header is fixed.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).
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
impl<__Context__> CanCalculateArea<__Context__> for RectangleArea
where
    __Context__: HasField<Symbol!("width"), Value = f64>
        + HasField<Symbol!("height"), Value = f64>,
{
    fn area(__context__: &__Context__) -> f64 {
        let width: f64 = ::core::clone::Clone::clone(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("width")>,
            ),
        );
        let height: f64 = ::core::clone::Clone::clone(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("height")>,
            ),
        );
        width * height
    }
}
impl<__Context__> IsProviderFor<CanCalculateAreaComponent, __Context__, ()>
for RectangleArea
where
    __Context__: HasField<Symbol!("width"), Value = f64>
        + HasField<Symbol!("height"), Value = f64>,
{}
///Provider `RectangleArea` for [`CanCalculateArea`].
pub struct RectangleArea;
pub struct Rectangle {
    pub width: f64,
    pub height: i32,
}
impl HasField<Symbol!("width")> for Rectangle {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &Self::Value {
        &self.width
    }
}
impl HasFieldMut<Symbol!("width")> for Rectangle {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &mut Self::Value {
        &mut self.width
    }
}
impl HasField<Symbol!("height")> for Rectangle {
    type Value = i32;
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
