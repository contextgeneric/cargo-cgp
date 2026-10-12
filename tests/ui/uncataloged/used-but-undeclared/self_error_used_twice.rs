//! A provider naming `Self::Error` twice without importing the abstract error type.
//!
//! rustc reports one `E0220` per use, each "associated type `Error` not found for `__Context__`".
//! One import, `#[use_type(HasErrorType.Error)]`, fixes both, so the tool reports one
//! `[CGP-E035]` at the first use and drops the second.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.

//@quality: acceptable
//@codes: CGP-E035

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
