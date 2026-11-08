#![feature(prelude_import)]
//! `Self::Error` in a component's method signature without importing the abstract error type.
//!
//! The signature is copied into the consumer trait, where `Self` is the context, and into the
//! provider trait, where `#[cgp_component]` writes it as `__Context__`, so rustc reports the same
//! `E0220` twice at one caret, once for `Self` and once for `__Context__`. The tool reports one
//! `[CGP-E035]` with the `#[use_type(HasErrorType.Error)]` import, and drops the copy at the same
//! caret.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanLoad {
    fn load(&self) -> Result<String, Self::Error>;
}
impl<__Context__> CanLoad for __Context__
where
    __Context__: Loader<__Context__>,
{
    fn load(&self) -> Result<String, Self::Error> {
        __Context__::load(self)
    }
}
pub trait Loader<__Context__>: IsProviderFor<LoaderComponent, __Context__, ()> {
    fn load(__context__: &__Context__) -> Result<String, __Context__::Error>;
}
impl<__Provider__, __Context__> Loader<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<LoaderComponent>
        + IsProviderFor<LoaderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<LoaderComponent>>::Delegate: Loader<__Context__>,
{
    fn load(__context__: &__Context__) -> Result<String, __Context__::Error> {
        <__Provider__ as DelegateComponent<LoaderComponent>>::Delegate::load(__context__)
    }
}
///Component marker for [`CanLoad`].
pub struct LoaderComponent;
impl<__Context__> Loader<__Context__> for UseContext
where
    __Context__: CanLoad,
{
    fn load(__context__: &__Context__) -> Result<String, __Context__::Error> {
        __Context__::load(__context__)
    }
}
impl<__Context__> IsProviderFor<LoaderComponent, __Context__, ()> for UseContext
where
    __Context__: CanLoad,
{}
impl<__Context__, __Components__, __Path__> Loader<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Loader<__Context__>,
{
    fn load(__context__: &__Context__) -> Result<String, __Context__::Error> {
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
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<LoaderComponent, __Context__, ()> + Loader<__Context__>,
{}
fn main() {}
