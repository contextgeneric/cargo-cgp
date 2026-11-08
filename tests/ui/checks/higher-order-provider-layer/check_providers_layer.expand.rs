#![feature(prelude_import)]
//! A `#[check_providers(...)]` per-layer assertion failing on the outer layer.
//!
//! The provider-side check form asserts `IsProviderFor` on each listed provider
//! directly, so a broken layer of a higher-order stack errors on its own line. The tool anchors
//! on the entry's generated check impl, whose `Self` is the listed provider, and walks the provider
//! obligation, so the block is the provider class `[CGP-E002]` and its tree starts at the failing
//! layer and runs to its missing field.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/higher-order-provider-layer.md.
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
pub trait HasScaleFactor {
    fn scale_factor(&self) -> f64;
}
impl<__Context__> HasScaleFactor for __Context__
where
    __Context__: HasField<Symbol!("scale_factor"), Value = f64>,
{
    fn scale_factor(&self) -> f64 {
        ::core::clone::Clone::clone(
            HasField::get_field(
                self,
                ::core::marker::PhantomData::<Symbol!("scale_factor")>,
            ),
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
impl<__Context__, InnerCalculator> AreaCalculator<__Context__>
for ScaledArea<InnerCalculator>
where
    __Context__: HasScaleFactor,
    InnerCalculator: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        __context__.scale_factor() * InnerCalculator::area(__context__)
    }
}
impl<
    __Context__,
    InnerCalculator,
> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for ScaledArea<InnerCalculator>
where
    __Context__: HasScaleFactor,
    InnerCalculator: IsProviderFor<AreaCalculatorComponent, __Context__, ()>
        + AreaCalculator<__Context__>,
{}
///Provider `ScaledArea` for [`AreaCalculator`].
pub struct ScaledArea<InnerCalculator>(pub ::core::marker::PhantomData<InnerCalculator>);
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
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
    type Delegate = ScaledArea<RectangleArea>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    ScaledArea<
        RectangleArea,
    >: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
trait CheckRectangleProviders<
    __Component__,
    __Context__,
    __Params__: ?Sized,
>: IsProviderFor<__Component__, __Context__, __Params__> {}
impl CheckRectangleProviders<AreaCalculatorComponent, Rectangle, ()> for RectangleArea {}
impl CheckRectangleProviders<AreaCalculatorComponent, Rectangle, ()>
for ScaledArea<RectangleArea> {}
fn main() {}
