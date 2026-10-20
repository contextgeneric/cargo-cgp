//! A qualified path the program wrote itself, `<Self as HasScalarType>::Scalr`, in a `#[cgp_fn]`.
//!
//! The path names an associated type `HasScalarType` does not declare, so the compiler reports
//! `E0576` with a help that respells `Scalr` to `Scalar` where it stands. No `#[use_type]` import is
//! involved, so that help is the fix and the tool keeps it.
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
#[uses(HasScalarType)]
pub fn get_scalar(&self) -> <Self as HasScalarType>::Scalr {
    todo!()
}

fn main() {}
