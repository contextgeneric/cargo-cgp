//! `Self::Error` in a `#[cgp_fn]` body without importing the abstract error type.
//!
//! A `#[cgp_fn]` body's `Self` is the generated blanket impl's context, which carries no
//! `HasErrorType` bound, so rustc reports `E0220`, "associated type `Error` not found for `Self`".
//! The tool reshapes it into `[CGP-E035]` with the `#[use_type(HasErrorType.Error)]` import in a
//! `help`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.

//@quality: acceptable
//@codes: CGP-E035

use cgp::prelude::*;

#[cgp_fn]
pub fn no_error(&self) -> bool {
    let _error: Option<Self::Error> = None;
    true
}

fn main() {}
