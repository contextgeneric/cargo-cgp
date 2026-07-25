#![feature(prelude_import)]
//! Usability: the same transitive-path burden as density_1, one layer deeper.
//!
//! Here `AreaCalculatorComponent` is `ScaledArea<RectangleArea>`, so the missing
//! `height` sits several hops below the checked `DensityCalculatorComponent`
//! (Density → Area → ScaledArea → RectangleArea → `height`). The extra higher-order
//! layer lengthens the `required for …` chain without adding a new cause, showing
//! that chain length tracks graph depth, not mistakes. The cause is recoverable, so
//! this is a usability problem: the summarized path must stay short even as the
//! graph deepens.
//!
//! Exposes issues in docs/issues/usability.md. CGP error class:
//! https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/verbose-cascade.md.
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
pub trait CanCalculateDensity {
    fn density(&self) -> f64;
}
impl<__Context__> CanCalculateDensity for __Context__
where
    __Context__: DensityCalculator<__Context__>,
{
    fn density(&self) -> f64 {
        __Context__::density(self)
    }
}
pub trait DensityCalculator<
    __Context__,
>: IsProviderFor<DensityCalculatorComponent, __Context__, ()> {
    fn density(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> DensityCalculator<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<DensityCalculatorComponent>
        + IsProviderFor<DensityCalculatorComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        DensityCalculatorComponent,
    >>::Delegate: DensityCalculator<__Context__>,
{
    fn density(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            DensityCalculatorComponent,
        >>::Delegate::density(__context__)
    }
}
pub struct DensityCalculatorComponent;
impl<__Context__> DensityCalculator<__Context__> for UseContext
where
    __Context__: CanCalculateDensity,
{
    fn density(__context__: &__Context__) -> f64 {
        __Context__::density(__context__)
    }
}
impl<__Context__> IsProviderFor<DensityCalculatorComponent, __Context__, ()>
for UseContext
where
    __Context__: CanCalculateDensity,
{}
impl<__Context__, __Components__, __Path__> DensityCalculator<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: DensityCalculator<__Context__>,
{
    fn density(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::density(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<DensityCalculatorComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<DensityCalculatorComponent, __Context__, ()>
        + DensityCalculator<__Context__>,
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
        self.get_field(::core::marker::PhantomData::<Symbol!("width")>).clone()
    }
    fn height(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("height")>).clone()
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
pub struct RectangleArea;
pub trait HasScaleFactor {
    fn scale_factor(&self) -> f64;
}
impl<__Context__> HasScaleFactor for __Context__
where
    __Context__: HasField<Symbol!("scale_factor"), Value = f64>,
{
    fn scale_factor(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("scale_factor")>).clone()
    }
}
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
pub struct ScaledArea<InnerCalculator>(pub ::core::marker::PhantomData<InnerCalculator>);
pub trait HasMass {
    fn mass(&self) -> f64;
}
impl<__Context__> HasMass for __Context__
where
    __Context__: HasField<Symbol!("mass"), Value = f64>,
{
    fn mass(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("mass")>).clone()
    }
}
impl<__Context__> DensityCalculator<__Context__> for DensityFromMassField
where
    __Context__: CanCalculateArea + HasMass,
{
    fn density(__context__: &__Context__) -> f64 {
        __context__.mass() / __context__.area()
    }
}
impl<__Context__> IsProviderFor<DensityCalculatorComponent, __Context__, ()>
for DensityFromMassField
where
    __Context__: CanCalculateArea + HasMass,
{}
pub struct DensityFromMassField;
pub struct Rectangle {
    pub mass: f64,
    pub width: f64,
}
impl HasField<Symbol!("mass")> for Rectangle {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("mass")>,
    ) -> &Self::Value {
        &self.mass
    }
}
impl HasFieldMut<Symbol!("mass")> for Rectangle {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("mass")>,
    ) -> &mut Self::Value {
        &mut self.mass
    }
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
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = ScaledArea<RectangleArea>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    ScaledArea<
        RectangleArea,
    >: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
impl DelegateComponent<DensityCalculatorComponent> for Rectangle {
    type Delegate = DensityFromMassField;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<DensityCalculatorComponent, __Context__, __Params__> for Rectangle
where
    DensityFromMassField: IsProviderFor<
        DensityCalculatorComponent,
        __Context__,
        __Params__,
    >,
{}
trait __CheckRectangle<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckRectangle<DensityCalculatorComponent, ()> for Rectangle {}
fn main() {}
