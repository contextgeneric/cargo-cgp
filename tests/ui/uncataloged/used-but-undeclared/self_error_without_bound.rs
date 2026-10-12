//! `Self::Error` in a provider without importing the abstract error type.
//!
//! `#[cgp_impl]` writes the body's `Self` as the generated `__Context__`, which carries no
//! `HasErrorType` bound, so rustc reports `E0220`, "associated type `Error` not found for
//! `__Context__`". The tool reshapes it into `[CGP-E035]`, naming the abstract type, with the
//! `#[use_type(HasErrorType.Error)]` import in a `help`.
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
        let _e: Option<Self::Error> = None;
        true
    }
}

fn main() {}
