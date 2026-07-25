#![feature(prelude_import)]
//! Two independent consumers coalesced on one cause via parallel (non-subsuming) chains.
//!
//! `CanCalculateArea` and `CanReportHeight` are unrelated — neither depends on the other
//! — yet both read the `height` field the `Rectangle` context is missing, so both fail
//! with the same root cause. cargo-cgp coalesces them into a single `[CGP-E001]` block
//! naming both consumers, with a caret per check entry and the shared cause shown once.
//!
//! Unlike `duplication/density_3.rs`, where `CanCalculateDensity`'s chain *subsumes*
//! `CanCalculateArea`'s (density depends on area) and the merged block must lead with the
//! deeper chain, here neither chain contains the other: they are equal-depth parallel
//! branches to the same missing field, each through its own getter and provider. With no
//! subsuming chain to prefer, the representative is the first check entry
//! (`AreaCalculatorComponent`) — a deterministic choice that does not flip on entry order.
//! This pins the parallel counterpart of the subsuming coalescing the cascade fixtures pin.
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
pub trait CanReportHeight {
    fn report_height(&self) -> f64;
}
impl<__Context__> CanReportHeight for __Context__
where
    __Context__: HeightReporter<__Context__>,
{
    fn report_height(&self) -> f64 {
        __Context__::report_height(self)
    }
}
pub trait HeightReporter<
    __Context__,
>: IsProviderFor<HeightReporterComponent, __Context__, ()> {
    fn report_height(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> HeightReporter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<HeightReporterComponent>
        + IsProviderFor<HeightReporterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        HeightReporterComponent,
    >>::Delegate: HeightReporter<__Context__>,
{
    fn report_height(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            HeightReporterComponent,
        >>::Delegate::report_height(__context__)
    }
}
pub struct HeightReporterComponent;
impl<__Context__> HeightReporter<__Context__> for UseContext
where
    __Context__: CanReportHeight,
{
    fn report_height(__context__: &__Context__) -> f64 {
        __Context__::report_height(__context__)
    }
}
impl<__Context__> IsProviderFor<HeightReporterComponent, __Context__, ()> for UseContext
where
    __Context__: CanReportHeight,
{}
impl<__Context__, __Components__, __Path__> HeightReporter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: HeightReporter<__Context__>,
{
    fn report_height(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::report_height(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<HeightReporterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<HeightReporterComponent, __Context__, ()>
        + HeightReporter<__Context__>,
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
pub trait HasHeight {
    fn height(&self) -> f64;
}
impl<__Context__> HasHeight for __Context__
where
    __Context__: HasField<Symbol!("height"), Value = f64>,
{
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
impl<__Context__> HeightReporter<__Context__> for ReportHeight
where
    __Context__: HasHeight,
{
    fn report_height(__context__: &__Context__) -> f64 {
        __context__.height()
    }
}
impl<__Context__> IsProviderFor<HeightReporterComponent, __Context__, ()>
for ReportHeight
where
    __Context__: HasHeight,
{}
pub struct ReportHeight;
pub struct Rectangle {
    pub width: f64,
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
impl DelegateComponent<HeightReporterComponent> for Rectangle {
    type Delegate = ReportHeight;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<HeightReporterComponent, __Context__, __Params__> for Rectangle
where
    ReportHeight: IsProviderFor<HeightReporterComponent, __Context__, __Params__>,
{}
trait __CheckRectangle<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}
impl __CheckRectangle<HeightReporterComponent, ()> for Rectangle {}
fn main() {}
