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
//! `Scalr` in place and fails with `E0425` on `Scalar`, so the tool drops it and keeps rustc's
//! header, with a `help` naming the rename in the import and in each use of the alias.
//!
//! CGP error class: unresolved imported abstract type (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/unresolved-imported-type.md).

//@quality: acceptable
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
