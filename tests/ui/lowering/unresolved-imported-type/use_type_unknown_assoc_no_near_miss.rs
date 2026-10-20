//! `#[use_type]` imports an associated type name the owning trait does not declare, and nothing it
//! declares is spelled close to it.
//!
//! `HasScalarType` declares only `Scalar`, but the import names `Weight`, so the bare `Weight` in
//! the signature is rewritten to `<Self as HasScalarType>::Weight` and the compiler reports `E0576`
//! at it. rustc suggests the trait's one associated type in the signature, which leaves the
//! import's `Weight` in place, so the edit does not compile. The tool keeps that suggestion: it
//! names the rename in the import only for a name that is a slip of a declared type.
//!
//! CGP error class: unresolved imported abstract type (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/unresolved-imported-type.md).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

#[cgp_fn]
#[use_type(HasScalarType.Weight)]
pub fn get_weight(&self) -> Weight {
    todo!()
}

fn main() {}
