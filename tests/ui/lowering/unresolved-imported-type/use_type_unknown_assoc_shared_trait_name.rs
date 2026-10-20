//! A misspelled `#[use_type]` import whose trait shares its name with another trait.
//!
//! `HasScalarType` declares `Scalar`, and the import names `Scalr`, as in
//! `use_type_unknown_assoc`. A second trait named `HasScalarType` sits in module `plain`, so the
//! name in the compiler's message does not say which trait the import names. The compiler's note on
//! the similarly named `Scalar` points at the trait it resolved, so the tool takes that trait,
//! drops the helps that respell the alias, and names the rename in the import.
//!
//! CGP error class: unresolved imported abstract type (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/unresolved-imported-type.md).

//@quality: acceptable
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
