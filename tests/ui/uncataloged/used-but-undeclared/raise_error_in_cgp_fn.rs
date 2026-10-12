//! A `#[cgp_fn]` body calling `Self::raise_error("failed")` without declaring `CanRaiseError`.
//!
//! In a `#[cgp_fn]` body `Self` is the generated blanket impl's `__Context__`, so rustc reports
//! `E0599`, no `raise_error` found for the type parameter `__Context__`, with suggestions spliced
//! after the `#[cgp_fn]` attribute. The argument is a string literal, whose reference type cannot be
//! written in `#[uses(…)]` without a lifetime, so the dependency's parameter is left to the reader.
//!
//! CGP error class: a used-but-undeclared dependency, not yet in the catalog
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md#a-gap-the-used-but-undeclared-and-wrong-half-classes).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_fn]
#[use_type(HasErrorType.Error)]
pub fn fail(&self) -> Error {
    Self::raise_error("failed")
}

fn main() {}
