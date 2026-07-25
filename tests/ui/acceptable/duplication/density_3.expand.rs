#![feature(prelude_import)]
//! One root cause reported once, listing every affected component.
//!
//! Both `AreaCalculatorComponent` and `DensityCalculatorComponent` are checked, and a
//! single missing `height` field breaks both. Left to rustc that is two full `E0277`
//! blocks — an error count reflecting the depth of the wiring graph, not the number of
//! mistakes. cargo-cgp coalesces the two: they share one root cause, so they collapse
//! into a single `[CGP-E001]` headline naming both consumer traits, with one caret per
//! failing check entry and the shared root cause shown once. This is the *different
//! consumers, one cause* coalescing — distinct from the same-consumer de-duplication
//! `duplication/cross_site_dedup.rs` pins.
//!
//! `DensityCalculator` depends on `CanCalculateArea`, so `CanCalculateDensity`'s chain
//! down to the missing field *contains* `CanCalculateArea`'s whole chain as a subtree.
//! The merged block leads with that deeper, subsuming chain — showing why the two merged
//! — even though the shallower `AreaCalculatorComponent` is the first entry checked; the
//! representative chain is the deepest, not the first to arrive.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/verbose-cascade.md.
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
    type Delegate = RectangleArea;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
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
impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}
impl __CheckRectangle<DensityCalculatorComponent, ()> for Rectangle {}
fn main() {}
