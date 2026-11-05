#![feature(prelude_import)]
//! A type mismatch in a provider body, in a crate that also holds an unrelated `E0107`.
//!
//! `LoadOrFail` returns a `String` where the context's abstract `Error` belongs (`E0308`), and an
//! unrelated function names `Vec` without its argument (`E0107`). Both are plain Rust errors, and
//! neither is a `#[cgp_impl]` header mistake, so the tool must print both exactly as rustc does:
//! the `E0107` elsewhere in the crate must not make the body's own error disappear.
//!
//! CGP error class: none; plain Rust the tool passes through.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanLoad: HasErrorType {
    fn load(&self) -> Result<u32, <Self as HasErrorType>::Error>;
}
impl<__Context__> CanLoad for __Context__
where
    __Context__: HasErrorType,
    __Context__: Loader<__Context__>,
{
    fn load(&self) -> Result<u32, <Self as HasErrorType>::Error> {
        __Context__::load(self)
    }
}
pub trait Loader<__Context__>: IsProviderFor<LoaderComponent, __Context__, ()>
where
    __Context__: HasErrorType,
{
    fn load(
        __context__: &__Context__,
    ) -> Result<u32, <__Context__ as HasErrorType>::Error>;
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
    ) -> Result<u32, <__Context__ as HasErrorType>::Error> {
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
    ) -> Result<u32, <__Context__ as HasErrorType>::Error> {
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
    ) -> Result<u32, <__Context__ as HasErrorType>::Error> {
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
    __Context__: CanRaiseError<String>,
    __Context__: HasErrorType,
{
    fn load(
        __context__: &__Context__,
    ) -> Result<u32, <__Context__ as HasErrorType>::Error> {
        Err("failed".to_owned())
    }
}
impl<__Context__> IsProviderFor<LoaderComponent, __Context__, ()> for LoadOrFail
where
    __Context__: CanRaiseError<String>,
    __Context__: HasErrorType,
{}
pub struct LoadOrFail;
pub fn unrelated(_: Vec) {}
fn main() {}
