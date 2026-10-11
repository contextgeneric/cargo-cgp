#![feature(prelude_import)]
//! A provider calling `Self::raise_error(…)` without declaring `CanRaiseError` in `#[uses]`.
//!
//! `Self` lowers to the provider impl's generated `__Context__` parameter, which the `[CGP-E016]`
//! inner-provider check skips, so the tool keeps rustc's `E0599`: no `raise_error` found for the
//! type parameter `__Context__`. It states nothing false, but it leaks `__Context__` and suggests a
//! bound inside `#[use_type]`; the fix is `#[uses(CanRaiseError<String>)]`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
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
impl<__Context__> Loader<__Context__> for LoadOrFail
where
    __Context__: HasErrorType,
{
    fn load(
        __context__: &__Context__,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        Err(__Context__::raise_error("failed".to_owned()))
    }
}
impl<__Context__> IsProviderFor<LoaderComponent, __Context__, ()> for LoadOrFail
where
    __Context__: HasErrorType,
{}
pub struct LoadOrFail;
fn main() {}
