#![feature(prelude_import)]
//! A `#[cgp_fn]` body calling `Self::raise_error("failed")` without declaring `CanRaiseError`.
//!
//! In a `#[cgp_fn]` body `Self` is the generated blanket impl's `__Context__`, so rustc reports
//! `E0599`, no `raise_error` found for the type parameter `__Context__`, with suggestions spliced
//! after the `#[cgp_fn]` attribute. The tool reshapes it into `[CGP-E030]`. The argument is a
//! string literal, whose reference type a `#[uses(…)]` bound cannot write without a lifetime, so the
//! help names `#[uses(CanRaiseError<SourceError>)]` and asks for the type in place of
//! `SourceError`.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait Fail: HasErrorType {
    fn fail(&self) -> <Self as HasErrorType>::Error;
}
impl<__Context__> Fail for __Context__
where
    Self: HasErrorType,
{
    fn fail(&self) -> <Self as HasErrorType>::Error {
        Self::raise_error("failed")
    }
}
fn main() {}
