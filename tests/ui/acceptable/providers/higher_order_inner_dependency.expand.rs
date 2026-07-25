#![feature(prelude_import)]
//! Acceptable failure: a higher-order provider whose *inner* layer carries the
//! unmet dependency. `ScaledArea<Inner>` delegates to an inner `AreaCalculator`
//! and adds its own `Self: HasScaleFactor` dependency; `BaseArea` is that inner
//! provider and needs `Self: HasBaseArea`. `Rectangle` supplies `scale_factor`
//! but not `base_area`, so the *outer* layer's dependency holds and the *inner*
//! layer's fails.
//!
//! This fixture pins where the diagnostic locates the failing layer: the
//! `unsatisfied trait bound introduced here` caret lands on `BaseArea`'s
//! `Self: HasBaseArea` clause (the inner provider), and the `required for …` chain
//! runs *through* `ScaledArea<BaseArea>`'s `IsProviderFor` before reaching
//! `CanUseComponent`, so the outer wrapper appears in the chain even though its own
//! bound is satisfied. Contrast higher_order_outer_dependency.rs, whose caret lands
//! on `ScaledArea`'s own clause and whose chain never reaches the inner provider —
//! the two failures are structurally similar but point at different layers, which
//! is why `#[check_providers(...)]` exists to assert `IsProviderFor` per layer.
//! This is the check doing its job, not a macro defect.
//!
//! See cgp-knowledge-base/cgp/errors/checks/higher-order-provider-layer.md.
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
pub trait HasBaseArea {
    fn base_area(&self) -> f64;
}
impl<__Context__> HasBaseArea for __Context__
where
    __Context__: HasField<Symbol!("base_area"), Value = f64>,
{
    fn base_area(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("base_area")>).clone()
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
        self.get_field(::core::marker::PhantomData::<Symbol!("scale_factor")>).clone()
    }
}
impl<__Context__> AreaCalculator<__Context__> for BaseArea
where
    __Context__: HasBaseArea,
{
    fn area(__context__: &__Context__) -> f64 {
        __context__.base_area()
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for BaseArea
where
    __Context__: HasBaseArea,
{}
pub struct BaseArea;
impl<__Context__, Inner> AreaCalculator<__Context__> for ScaledArea<Inner>
where
    __Context__: HasScaleFactor,
    Inner: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        __context__.scale_factor() * Inner::area(__context__)
    }
}
impl<__Context__, Inner> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for ScaledArea<Inner>
where
    __Context__: HasScaleFactor,
    Inner: IsProviderFor<AreaCalculatorComponent, __Context__, ()>
        + AreaCalculator<__Context__>,
{}
pub struct ScaledArea<Inner>(pub ::core::marker::PhantomData<Inner>);
pub struct Rectangle {
    pub scale_factor: f64,
}
impl HasField<Symbol!("scale_factor")> for Rectangle {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("scale_factor")>,
    ) -> &Self::Value {
        &self.scale_factor
    }
}
impl HasFieldMut<Symbol!("scale_factor")> for Rectangle {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("scale_factor")>,
    ) -> &mut Self::Value {
        &mut self.scale_factor
    }
}
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = ScaledArea<BaseArea>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    ScaledArea<
        BaseArea,
    >: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
trait __CheckRectangle<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}
fn main() {}
