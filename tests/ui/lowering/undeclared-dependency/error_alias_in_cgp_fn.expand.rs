#![feature(prelude_import)]
//! A bare `Error` in a `#[cgp_fn]` signature without `#[use_type(HasErrorType.Error)]`.
//!
//! Without the import nothing rewrites `Error` to the context's abstract type, so name resolution
//! reports `E0425` "cannot find type `Error` in this scope" and suggests importing
//! `std::error::Error`, `std::fmt::Error`, or `std::io::Error`, none of which is the fix. The error
//! arrives before the compiler can tell which trait declares an `Error`, and no requirement of a
//! component names one here, so the tool leaves it as rustc wrote it.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
///The trait for the `some_error` operation.
pub trait SomeError {
    ///The `some_error` operation.
    fn some_error(&self) -> Option<Error>;
}
impl<__Context__> SomeError for __Context__ {
    fn some_error(&self) -> Option<Error> {
        None
    }
}
fn main() {}
