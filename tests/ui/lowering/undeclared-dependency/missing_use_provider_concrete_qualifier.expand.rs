#![feature(prelude_import)]
//! A provider body calls another provider by its own path, `WidthArea::area(self)`, without declaring
//! the requirement that provider has of the context (`#[uses(HasWidth)]`).
//!
//! The tool leaves rustc's `E0277` on `__Context__: HasField<Symbol!("width")>`, which names the
//! generated context and the field the called provider reads rather than the declaration the
//! calling provider lacks. A concrete provider is not an inner provider, so `[CGP-E031]` does not
//! claim the call.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
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
impl<__Context__> AreaCalculator<__Context__> for WidthArea
where
    __Context__: HasWidth,
{
    fn area(__context__: &__Context__) -> f64 {
        __context__.width()
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for WidthArea
where
    __Context__: HasWidth,
{}
pub struct WidthArea;
impl<__Context__> AreaCalculator<__Context__> for DoubledArea {
    fn area(__context__: &__Context__) -> f64 {
        WidthArea::area(__context__) * 2.0
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for DoubledArea {}
pub struct DoubledArea;
fn main() {}
