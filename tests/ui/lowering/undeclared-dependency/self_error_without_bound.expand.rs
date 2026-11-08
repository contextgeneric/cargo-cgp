#![feature(prelude_import)]
//! `Self::Error` in a provider without importing the abstract error type.
//!
//! `#[cgp_impl]` writes the body's `Self` as the generated `__Context__`, which carries no
//! `HasErrorType` bound, so rustc reports `E0220`, "associated type `Error` not found for
//! `__Context__`". The tool reshapes it into `[CGP-E035]`, naming the abstract type, with the
//! `#[use_type(HasErrorType.Error)]` import in a `help`.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
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
///Component marker for [`CanValidate`].
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
        let _e: Option<__Context__::Error> = None;
        true
    }
}
impl<__Context__> IsProviderFor<ValidatorComponent, __Context__, ()>
for ValidateNothing {}
///Provider `ValidateNothing` for [`Validator`].
pub struct ValidateNothing;
fn main() {}
