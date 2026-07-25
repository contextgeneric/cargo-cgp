#![feature(prelude_import)]
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
impl<__Context__> AreaCalculator<__Context__> for RectangleArea
where
    __Context__: HasField<Symbol!("width"), Value = f64>
        + HasField<Symbol!("height"), Value = f64>,
{
    fn area(__context__: &__Context__) -> f64 {
        let width: f64 = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("width")>)
            .clone();
        let height: f64 = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("height")>)
            .clone();
        width * height
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for RectangleArea
where
    __Context__: HasField<Symbol!("width"), Value = f64>
        + HasField<Symbol!("height"), Value = f64>,
{}
pub struct RectangleArea;
impl<__Context__, InnerCalculator> AreaCalculator<__Context__>
for ScaledArea<InnerCalculator>
where
    __Context__: HasField<Symbol!("scale_factor"), Value = f64>,
    InnerCalculator: CanCalculateArea<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        let scale_factor: f64 = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("scale_factor")>)
            .clone();
        InnerCalculator::area(__context__) * scale_factor * scale_factor
    }
}
impl<
    __Context__,
    InnerCalculator,
> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for ScaledArea<InnerCalculator>
where
    __Context__: HasField<Symbol!("scale_factor"), Value = f64>,
    InnerCalculator: CanCalculateArea<__Context__>,
{}
pub struct ScaledArea<InnerCalculator>(pub ::core::marker::PhantomData<InnerCalculator>);
fn main() {}
