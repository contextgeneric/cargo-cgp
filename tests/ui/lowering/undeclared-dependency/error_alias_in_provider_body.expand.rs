#![feature(prelude_import)]
//! A bare `Error` in a `#[cgp_impl]` method body without `#[use_type(HasErrorType.Error)]`.
//!
//! The component requires nothing of the context, so the only error is name resolution's `E0425`
//! "cannot find type `Error` in this scope", with suggestions to import `std::error::Error` and its
//! namesakes, none of which is the fix. The tool rewords a bare `Error` only when a requirement of
//! the component names the trait that declares it (`error_alias_without_use_type`), so it leaves
//! this one as rustc wrote it.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
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
        let _error: Option<Error> = None;
        true
    }
}
impl<__Context__> IsProviderFor<ValidatorComponent, __Context__, ()>
for ValidateNothing {}
pub struct ValidateNothing;
fn main() {}
