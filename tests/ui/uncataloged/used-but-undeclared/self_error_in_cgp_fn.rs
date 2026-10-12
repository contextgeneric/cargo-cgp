//! `Self::Error` in a `#[cgp_fn]` body without importing the abstract error type.
//!
//! A `#[cgp_fn]` body's `Self` is the generated blanket impl's context, which carries no
//! `HasErrorType` bound, so rustc reports `E0220`, "associated type `Error` not found for `Self`",
//! naming no fix. The fix is `#[use_type(HasErrorType.Error)]` on the function.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_fn]
pub fn no_error(&self) -> bool {
    let _error: Option<Self::Error> = None;
    true
}

fn main() {}
