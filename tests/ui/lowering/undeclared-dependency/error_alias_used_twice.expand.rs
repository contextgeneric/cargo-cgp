#![feature(prelude_import)]
//! A provider writing the abstract `Error` twice without its own `#[use_type(HasErrorType.Error)]`,
//! beside a function outside the provider that names an undeclared `Error` too.
//!
//! rustc reports an `E0425` on each `Error`, and two `E0277`s on `__Context__`, since the
//! component's `#[use_type]` requires `HasErrorType` of every context. The tool reports one
//! `[CGP-E035]` on the provider's first `Error` and drops its second and the two `E0277`s, which the
//! one import fixes. The `Error` in `describe` is outside the provider, so its `E0425` stays as
//! rustc wrote it.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanLoad: HasErrorType {
    fn load(&self) -> Result<String, <Self as HasErrorType>::Error>;
    fn reload(&self) -> Result<String, <Self as HasErrorType>::Error>;
}
impl<__Context__> CanLoad for __Context__
where
    __Context__: HasErrorType,
    __Context__: Loader<__Context__>,
{
    fn load(&self) -> Result<String, <Self as HasErrorType>::Error> {
        __Context__::load(self)
    }
    fn reload(&self) -> Result<String, <Self as HasErrorType>::Error> {
        __Context__::reload(self)
    }
}
pub trait Loader<__Context__>: IsProviderFor<LoaderComponent, __Context__, ()>
where
    __Context__: HasErrorType,
{
    fn load(
        __context__: &__Context__,
    ) -> Result<String, <__Context__ as HasErrorType>::Error>;
    fn reload(
        __context__: &__Context__,
    ) -> Result<String, <__Context__ as HasErrorType>::Error>;
}
impl<__Provider__, __Context__> Loader<__Context__> for __Provider__
where
    __Context__: HasErrorType,
    __Provider__: DelegateComponent<LoaderComponent>
        + IsProviderFor<LoaderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<LoaderComponent>>::Delegate: Loader<__Context__>,
{
    fn load(
        __context__: &__Context__,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        <__Provider__ as DelegateComponent<LoaderComponent>>::Delegate::load(__context__)
    }
    fn reload(
        __context__: &__Context__,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        <__Provider__ as DelegateComponent<
            LoaderComponent,
        >>::Delegate::reload(__context__)
    }
}
pub struct LoaderComponent;
impl<__Context__> Loader<__Context__> for UseContext
where
    __Context__: HasErrorType,
    __Context__: CanLoad,
{
    fn load(
        __context__: &__Context__,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        __Context__::load(__context__)
    }
    fn reload(
        __context__: &__Context__,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        __Context__::reload(__context__)
    }
}
impl<__Context__> IsProviderFor<LoaderComponent, __Context__, ()> for UseContext
where
    __Context__: HasErrorType,
    __Context__: CanLoad,
{}
impl<__Context__, __Components__, __Path__> Loader<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Loader<__Context__>,
{
    fn load(
        __context__: &__Context__,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::load(__context__)
    }
    fn reload(
        __context__: &__Context__,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::reload(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<LoaderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<LoaderComponent, __Context__, ()> + Loader<__Context__>,
{}
pub fn describe(_error: Error) {}
impl<__Context__> Loader<__Context__> for LoadNothing {
    fn load(__context__: &__Context__) -> Result<String, Error> {
        Ok(String::new())
    }
    fn reload(__context__: &__Context__) -> Result<String, Error> {
        Ok(String::new())
    }
}
impl<__Context__> IsProviderFor<LoaderComponent, __Context__, ()> for LoadNothing {}
pub struct LoadNothing;
fn main() {}
