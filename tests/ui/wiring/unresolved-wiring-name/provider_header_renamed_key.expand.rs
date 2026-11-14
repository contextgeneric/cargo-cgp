#![feature(prelude_import)]
//! A provider header for a component whose `name:` renames its key.
//!
//! The component's key is `AreaComponent`, but the header `impl AreaCalculator` makes the macro name
//! it `AreaCalculatorComponent`, which nothing declares, so rustc reports "cannot find type
//! `AreaCalculatorComponent`" with no suggestion. The tool reshapes it into `[CGP-E070]`, naming
//! the real key and the attribute form that names it.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
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
///Provider trait for [`CanCalculateArea`]. A provider imports this trait's component key, [`AreaComponent`], beside it.
pub trait AreaCalculator<__Context__>: IsProviderFor<AreaComponent, __Context__, ()> {
    fn area(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> AreaCalculator<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<AreaComponent>
        + IsProviderFor<AreaComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        AreaComponent,
    >>::Delegate: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<AreaComponent>>::Delegate::area(__context__)
    }
}
///Component marker for [`CanCalculateArea`].
pub struct AreaComponent;
impl<__Context__> AreaCalculator<__Context__> for UseContext
where
    __Context__: CanCalculateArea,
{
    fn area(__context__: &__Context__) -> f64 {
        __Context__::area(__context__)
    }
}
impl<__Context__> IsProviderFor<AreaComponent, __Context__, ()> for UseContext
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
impl<__Context__, __Components__, __Path__> IsProviderFor<AreaComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<AreaComponent, __Context__, ()>
        + AreaCalculator<__Context__>,
{}
impl<__Context__> AreaCalculator<__Context__> for UnitArea {
    fn area(__context__: &__Context__) -> f64 {
        1.0
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for UnitArea {}
///Provider `UnitArea` for [`AreaCalculator`].
pub struct UnitArea;
fn main() {}
