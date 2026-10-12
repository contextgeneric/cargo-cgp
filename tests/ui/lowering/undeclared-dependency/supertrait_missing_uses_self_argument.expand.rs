#![feature(prelude_import)]
//! A provider for a component with `#[extend(CanCompareWith<Self>)]` that omits the matching
//! `#[uses]`, where the requirement's argument is the context itself.
//!
//! The provider trait requires `__Context__: CanCompareWith<__Context__>`, and the provider declares
//! nothing, so rustc reports `E0277` on that bound. The tool reshapes it into `[CGP-E036]`, but the
//! requirement is written as the compiler sees it, with the generated `__Context__` in the header
//! and in the `#[uses(CanCompareWith<__Context__>)]` help, where the provider writes `Self`.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanCompareWith<Other: ?Sized> {
    fn compare_with(&self, other: &Other) -> bool;
}
impl<T: PartialEq> CanCompareWith<T> for T {
    fn compare_with(&self, other: &T) -> bool {
        self == other
    }
}
pub trait CanCheck: CanCompareWith<Self> {
    fn check(&self) -> bool;
}
impl<__Context__> CanCheck for __Context__
where
    __Context__: CanCompareWith<Self>,
    __Context__: Checker<__Context__>,
{
    fn check(&self) -> bool {
        __Context__::check(self)
    }
}
pub trait Checker<__Context__>: IsProviderFor<CheckerComponent, __Context__, ()>
where
    __Context__: CanCompareWith<__Context__>,
{
    fn check(__context__: &__Context__) -> bool;
}
impl<__Provider__, __Context__> Checker<__Context__> for __Provider__
where
    __Context__: CanCompareWith<__Context__>,
    __Provider__: DelegateComponent<CheckerComponent>
        + IsProviderFor<CheckerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        CheckerComponent,
    >>::Delegate: Checker<__Context__>,
{
    fn check(__context__: &__Context__) -> bool {
        <__Provider__ as DelegateComponent<
            CheckerComponent,
        >>::Delegate::check(__context__)
    }
}
pub struct CheckerComponent;
impl<__Context__> Checker<__Context__> for UseContext
where
    __Context__: CanCompareWith<__Context__>,
    __Context__: CanCheck,
{
    fn check(__context__: &__Context__) -> bool {
        __Context__::check(__context__)
    }
}
impl<__Context__> IsProviderFor<CheckerComponent, __Context__, ()> for UseContext
where
    __Context__: CanCompareWith<__Context__>,
    __Context__: CanCheck,
{}
impl<__Context__, __Components__, __Path__> Checker<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: CanCompareWith<__Context__>,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Checker<__Context__>,
{
    fn check(__context__: &__Context__) -> bool {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::check(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<CheckerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: CanCompareWith<__Context__>,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<CheckerComponent, __Context__, ()>
        + Checker<__Context__>,
{}
impl<__Context__> Checker<__Context__> for CheckSelf {
    fn check(__context__: &__Context__) -> bool {
        true
    }
}
impl<__Context__> IsProviderFor<CheckerComponent, __Context__, ()> for CheckSelf {}
pub struct CheckSelf;
fn main() {}
