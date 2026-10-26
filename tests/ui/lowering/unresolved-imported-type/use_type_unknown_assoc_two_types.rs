//! `#[use_type]` imports a misspelled associated type from a trait that declares two.
//!
//! `HasTypes` declares `Scalar` and `Unit`, and the import names `Scalr`, so the substituted
//! `<Self as HasTypes>::Scalr` names no associated type and the compiler rejects it with `E0576`.
//! With more than one associated type, the tool names the rename only for the one whose name is
//! close to the misspelled one, `Scalar`, in a `help` that covers the import and each use.
//!
//! CGP error class: unresolved imported abstract type (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/unresolved-imported-type.md).

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

pub trait HasTypes {
    type Scalar;
    type Unit;
}

#[cgp_fn]
#[use_type(HasTypes.Scalr)]
pub fn get_scalar(&self) -> Scalr {
    todo!()
}

fn main() {}
