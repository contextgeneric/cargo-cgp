//! A misspelled `#[use_type]` import whose trait shares its name with another trait.
//!
//! `HasScalarType` declares `Scalar`, and the import names `Scalr`, as in
//! `use_type_unknown_assoc`. A second trait named `HasScalarType` sits in module `plain`, so the
//! name in the compiler's message does not say which trait the import names, and the tool keeps
//! the compiler's output rather than guess.
//!
//! CGP error class: unresolved imported abstract type (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/unresolved-imported-type.md).

//@quality: usability
//@codes: none

use cgp::prelude::*;

pub mod plain {
    pub trait HasScalarType {
        type Scalar;
    }
}

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
