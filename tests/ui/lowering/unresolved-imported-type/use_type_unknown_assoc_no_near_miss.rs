//! `#[use_type]` imports an associated type name the owning trait does not declare, and nothing it
//! declares is spelled close to it.
//!
//! `HasScalarType` declares only `Scalar`, but the import names `Weight`, so the bare `Weight` in
//! the signature is rewritten to `<Self as HasScalarType>::Weight` and the compiler reports `E0576`
//! at it. rustc suggests the trait's one associated type in the signature, which leaves the
//! import's `Weight` in place. That type is the one name an import of the trait can give, so the
//! tool drops the suggestion and names the rename in the import instead.
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
#[use_type(HasScalarType.Weight)]
pub fn get_weight(&self) -> Weight {
    todo!()
}

fn main() {}
