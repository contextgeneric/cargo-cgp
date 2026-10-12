//! A provider naming `Self::Error` twice without importing the abstract error type.
//!
//! rustc reports one `E0220` per use, each "associated type `Error` not found for `__Context__`",
//! though one import, `#[use_type(HasErrorType.Error)]`, fixes both.
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
        let _first: Option<Self::Error> = None;
        let _second: Option<Self::Error> = None;
        true
    }
}

fn main() {}
