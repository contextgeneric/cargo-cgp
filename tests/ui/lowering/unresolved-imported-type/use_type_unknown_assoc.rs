//! `#[use_type]` imports an associated type name the owning trait does not declare, so the
//! substituted `<Self as Trait>::WrongName` path names an associated type that does not exist and
//! the compiler rejects it.
//!
//! `HasScalarType` declares `Scalar`, but the import names `Scalr` (a typo), so the bare `Scalr` in
//! the signature is rewritten to `<Self as HasScalarType>::Scalr`. CGP cannot know the trait's
//! associated types at expansion time, since it performs a textual rewrite, so it lowers the name
//! faithfully and defers to the compiler, which reports `E0576` "cannot find associated type
//! `Scalr`". The substitution preserves the span of the identifier the user wrote, so the caret
//! lands on the `Scalr` in the signature, not on the macro attribute, and this fixture also guards
//! that span behavior. rustc's suggestion respells the signature only, which leaves the attribute's
//! `Scalr` in place and fails with `E0425` on `Scalar`; the tool passes the error through.
//!
//! CGP error class: unresolved imported abstract type (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/unresolved-imported-type.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

#[cgp_fn]
#[use_type(HasScalarType.Scalr)]
pub fn get_scalar(&self) -> Scalr {
    todo!()
}

fn main() {}
