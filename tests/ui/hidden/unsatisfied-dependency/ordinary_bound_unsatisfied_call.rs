//! The same unmet *ordinary Rust trait bound* dependency as
//! `ordinary_bound_unsatisfied` (`Scalar: Eq` with `f64` wired),
//! but exercised by calling the consumer method rather than a check, so the cause
//! is *hidden*.
//!
//! Calling `app.scalars_equal(..)` produces the `E0599` "method exists but its trait bounds were
//! not satisfied" shape: it names `App: CanCompareScalars` / `App: ScalarEquality<App>`,
//! misclassifies the method as an associated function (the provider method has no `self` receiver),
//! and suggests `App::scalars_equal()` but never mentions the unmet `f64: Eq`. This is
//! byte-for-shape identical to the HasName hidden case in
//! `hidden/unsatisfied-dependency/missing_dependency_call.rs`: the compiler's method-probe
//! heuristic drops the nested `where`-clause bound regardless of whether that bound is a
//! `HasField`, a CGP trait, or an ordinary trait. The tool's call-site block states the cause rustc
//! hid: a `[CGP-E001]` headline reading `` the trait bound `f64: Eq` is not satisfied `` over the
//! chain to the `[CGP-E117]` leaf.
//!
//! See cgp-knowledge-base/cgp/errors/hidden/unsatisfied-dependency.md; the surfaced counterpart is
//! cgp-knowledge-base/cgp/errors/checks/ordinary-trait-bound.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E117

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

#[cgp_component(ScalarEquality)]
#[use_type(HasScalarType.Scalar)]
pub trait CanCompareScalars {
    fn scalars_equal(&self, a: &Scalar, b: &Scalar) -> bool;
}

#[cgp_impl(new CompareScalars)]
#[use_type(HasScalarType.Scalar)]
impl ScalarEquality
where
    Scalar: Eq,
{
    fn scalars_equal(&self, a: &Scalar, b: &Scalar) -> bool {
        a == b
    }
}

pub struct App;

delegate_components! {
    App {
        ScalarTypeProviderComponent: UseType<f64>,
        ScalarEqualityComponent: CompareScalars,
    }
}

fn main() {
    let app = App;
    let _ = app.scalars_equal(&1.0, &2.0);
}
