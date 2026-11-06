//! `Self::Error` without an import, where two abstract-type traits declare an `Error`.
//!
//! The crate defines its own `#[cgp_type] trait HasErrorType` beside CGP's, so the abstract type
//! the provider means cannot be told from the name alone. The tool leaves rustc's `E0220` as it is
//! rather than guess which trait to import.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use cgp::prelude::*;

pub mod local {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasErrorType {
        type Error;
    }
}

#[cgp_component(Validator)]
pub trait CanValidate {
    fn validate(&self) -> bool;
}

#[cgp_impl(new ValidateNothing)]
impl Validator {
    fn validate(&self) -> bool {
        let _error: Option<Self::Error> = None;
        true
    }
}

fn main() {}
