#![feature(prelude_import)]
//! `Self::Error` in a `#[cgp_fn]` body without importing the abstract error type.
//!
//! A `#[cgp_fn]` body's `Self` is the generated blanket impl's context, which carries no
//! `HasErrorType` bound, so rustc reports `E0220`, "associated type `Error` not found for `Self`",
//! naming no fix. The fix is `#[use_type(HasErrorType.Error)]` on the function.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait NoError {
    fn no_error(&self) -> bool;
}
impl<__Context__> NoError for __Context__ {
    fn no_error(&self) -> bool {
        let _error: Option<Self::Error> = None;
        true
    }
}
fn main() {}
