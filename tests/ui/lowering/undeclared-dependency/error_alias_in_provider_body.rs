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
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Validator)]
pub trait CanValidate {
    fn validate(&self) -> bool;
}

#[cgp_impl(new ValidateNothing)]
impl Validator {
    fn validate(&self) -> bool {
        let _error: Option<Error> = None;
        true
    }
}

fn main() {}
