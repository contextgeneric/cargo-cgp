#![feature(prelude_import)]
//! `Self::Error` in a `#[cgp_fn]` body without importing the abstract error type.
//!
//! A `#[cgp_fn]` body's `Self` is the generated blanket impl's context, which carries no
//! `HasErrorType` bound, so rustc reports `E0220`, "associated type `Error` not found for `Self`".
//! The tool reshapes it into `[CGP-E035]` with the `#[use_type(HasErrorType.Error)]` import in a
//! `help`.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
///The trait for the `no_error` operation.
pub trait NoError {
    ///The `no_error` operation.
    fn no_error(&self) -> bool;
}
impl<__Context__> NoError for __Context__ {
    fn no_error(&self) -> bool {
        let _error: Option<Self::Error> = None;
        true
    }
}
fn main() {}
