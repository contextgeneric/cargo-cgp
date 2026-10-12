#![feature(prelude_import)]
//! A provider for a component with `#[use_type(HasErrorType.Error)]` that writes `Self::Error`
//! without importing it.
//!
//! rustc reports three errors for the one missing `#[use_type(HasErrorType.Error)]`: an `E0220` on
//! `Self::Error`, and two `E0277`s on `__Context__: HasErrorType` at the method's signature and at
//! the `#[cgp_impl]` attribute, since the component requires the trait of every context. The tool
//! reports one `[CGP-E035]` with the import, and drops the two `E0277`s, which the same import fixes.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanLoad: HasErrorType {
    fn load(&self) -> Result<String, <Self as HasErrorType>::Error>;
}
impl<__Context__> CanLoad for __Context__
where
    __Context__: HasErrorType,
    __Context__: Loader<__Context__>,
{
    fn load(&self) -> Result<String, <Self as HasErrorType>::Error> {
        __Context__::load(self)
    }
}
pub trait Loader<__Context__>: IsProviderFor<LoaderComponent, __Context__, ()>
where
    __Context__: HasErrorType,
{
    fn load(
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
impl<__Context__> Loader<__Context__> for LoadNothing {
    fn load(__context__: &__Context__) -> Result<String, __Context__::Error> {
        Ok(String::new())
    }
}
impl<__Context__> IsProviderFor<LoaderComponent, __Context__, ()> for LoadNothing {}
pub struct LoadNothing;
fn main() {}
