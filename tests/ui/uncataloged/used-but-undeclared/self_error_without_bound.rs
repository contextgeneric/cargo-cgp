//! `Self::Error` in a provider without importing the abstract error type.
//!
//! `E0220` "associated type `Error` not found for `__Context__`"; the fix is
//! `#[use_type(HasErrorType.Error)]`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
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
        let _e: Option<Self::Error> = None;
        true
    }
}

fn main() {}
