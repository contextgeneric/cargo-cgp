#![feature(prelude_import)]
//! A provider naming `Self::Error` twice without importing the abstract error type.
//!
//! rustc reports one `E0220` per use, each "associated type `Error` not found for `__Context__`",
//! though one import, `#[use_type(HasErrorType.Error)]`, fixes both.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanValidate {
    fn validate(&self) -> bool;
}
impl<__Context__> CanValidate for __Context__
where
    __Context__: Validator<__Context__>,
{
    fn validate(&self) -> bool {
        __Context__::validate(self)
    }
}
pub trait Validator<__Context__>: IsProviderFor<ValidatorComponent, __Context__, ()> {
    fn validate(__context__: &__Context__) -> bool;
}
impl<__Provider__, __Context__> Validator<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ValidatorComponent>
        + IsProviderFor<ValidatorComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ValidatorComponent,
    >>::Delegate: Validator<__Context__>,
{
    fn validate(__context__: &__Context__) -> bool {
        <__Provider__ as DelegateComponent<
            ValidatorComponent,
        >>::Delegate::validate(__context__)
    }
}
pub struct ValidatorComponent;
impl<__Context__> Validator<__Context__> for UseContext
where
    __Context__: CanValidate,
{
    fn validate(__context__: &__Context__) -> bool {
        __Context__::validate(__context__)
    }
}
impl<__Context__> IsProviderFor<ValidatorComponent, __Context__, ()> for UseContext
where
    __Context__: CanValidate,
{}
impl<__Context__, __Components__, __Path__> Validator<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Validator<__Context__>,
{
    fn validate(__context__: &__Context__) -> bool {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::validate(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ValidatorComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ValidatorComponent, __Context__, ()>
        + Validator<__Context__>,
{}
impl<__Context__> Validator<__Context__> for ValidateNothing {
    fn validate(__context__: &__Context__) -> bool {
        let _first: Option<__Context__::Error> = None;
        let _second: Option<__Context__::Error> = None;
        true
    }
}
impl<__Context__> IsProviderFor<ValidatorComponent, __Context__, ()>
for ValidateNothing {}
pub struct ValidateNothing;
fn main() {}
