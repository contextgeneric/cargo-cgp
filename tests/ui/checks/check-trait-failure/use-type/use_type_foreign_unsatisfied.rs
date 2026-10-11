//! A foreign `#[use_type(HasScalarType.Scalar in Types)]` import adds `Types: HasScalarType`
//! to the generated trait, so naming the component for a `Types` that does not implement
//! `HasScalarType` is rejected by the compiler.
//!
//! The bound keeps `NoScalar` from slipping through and failing much later, or not at all if
//! the abstract type goes unused. CGP emits the bound and defers the check to `rustc`, which
//! reports the missing `NoScalar: HasScalarType` at the use site.
//!
//! See cgp-knowledge-base/cgp/reference/attributes/use_type.md and
//! cgp-knowledge-base/cgp/errors/checks/check-trait-failure.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E107

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

#[cgp_component(AreaCalculator)]
#[use_type(HasScalarType.Scalar in Types)]
pub trait CanCalculateArea<Types> {
    fn area(&self) -> Scalar;
}

// `NoScalar` deliberately does not implement `HasScalarType`.
pub struct NoScalar;

// Asserting the component for `NoScalar` requires `NoScalar: HasScalarType`, which
// the foreign import now demands — so this fails to compile.
pub trait CheckMissingScalar: CanCalculateArea<NoScalar> {}

fn main() {}
