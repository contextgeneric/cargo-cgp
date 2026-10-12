#![feature(prelude_import)]
//! `Self::Error` without an import, where two abstract-type traits declare an `Error`.
//!
//! The crate defines its own `#[cgp_type] trait HasErrorType` beside CGP's, so the abstract type
//! the provider means cannot be told from the name alone. The tool leaves rustc's `E0220` as it is
//! rather than guess which trait to import.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub mod local {
    use cgp::prelude::*;
    pub trait HasErrorType {
        type Error;
    }
    impl<__Context__> HasErrorType for __Context__
    where
        __Context__: ErrorTypeProvider<__Context__>,
    {
        type Error = <__Context__ as ErrorTypeProvider<__Context__>>::Error;
    }
    pub trait ErrorTypeProvider<
        __Context__,
    >: IsProviderFor<ErrorTypeProviderComponent, __Context__, ()> {
        type Error;
    }
    impl<__Provider__, __Context__> ErrorTypeProvider<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<ErrorTypeProviderComponent>
            + IsProviderFor<ErrorTypeProviderComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            ErrorTypeProviderComponent,
        >>::Delegate: ErrorTypeProvider<__Context__>,
    {
        type Error = <<__Provider__ as DelegateComponent<
            ErrorTypeProviderComponent,
        >>::Delegate as ErrorTypeProvider<__Context__>>::Error;
    }
    pub struct ErrorTypeProviderComponent;
    impl<__Context__> ErrorTypeProvider<__Context__> for UseContext
    where
        __Context__: HasErrorType,
    {
        type Error = <__Context__ as HasErrorType>::Error;
    }
    impl<__Context__> IsProviderFor<ErrorTypeProviderComponent, __Context__, ()>
    for UseContext
    where
        __Context__: HasErrorType,
    {}
    impl<__Context__, __Components__, __Path__> ErrorTypeProvider<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: ErrorTypeProvider<__Context__>,
    {
        type Error = <<__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate as ErrorTypeProvider<__Context__>>::Error;
    }
    impl<
        __Context__,
        __Components__,
        __Path__,
    > IsProviderFor<ErrorTypeProviderComponent, __Context__, ()>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<ErrorTypeProviderComponent, __Context__, ()>
            + ErrorTypeProvider<__Context__>,
    {}
    impl<Error, __Context__> ErrorTypeProvider<__Context__> for UseType<Error> {
        type Error = Error;
    }
    impl<Error, __Context__> IsProviderFor<ErrorTypeProviderComponent, __Context__, ()>
    for UseType<Error> {}
    impl<__Provider__, Error, __Context__> ErrorTypeProvider<__Context__>
    for WithProvider<__Provider__>
    where
        __Provider__: TypeProvider<
            __Context__,
            ErrorTypeProviderComponent,
            Type = Error,
        >,
    {
        type Error = Error;
    }
    impl<
        __Provider__,
        Error,
        __Context__,
    > IsProviderFor<ErrorTypeProviderComponent, __Context__, ()>
    for WithProvider<__Provider__>
    where
        __Provider__: TypeProvider<
            __Context__,
            ErrorTypeProviderComponent,
            Type = Error,
        >,
    {}
}
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
        let _error: Option<__Context__::Error> = None;
        true
    }
}
impl<__Context__> IsProviderFor<ValidatorComponent, __Context__, ()>
for ValidateNothing {}
pub struct ValidateNothing;
fn main() {}
