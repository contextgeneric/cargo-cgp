//! `#[use_type]` imports a misspelled associated type under an `as` alias.
//!
//! `HasScalarType` declares `Scalar`, but `#[use_type(HasScalarType.{Scalr as Value})]` names
//! `Scalr`, so the bare `Value` in the signature is rewritten to `<Self as HasScalarType>::Scalr`
//! and the compiler reports `E0576` "cannot find associated type `Scalr`" at `Value`. The alias is
//! the program's own name, so only the import needs the rename.
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
#[use_type(HasScalarType.{Scalr as Value})]
pub fn get_scalar(&self) -> Value {
    todo!()
}

fn main() {}
