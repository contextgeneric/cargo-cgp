//! A provider for a component with `#[extend(CanCompareWith<Self>)]` that omits the matching
//! `#[uses]`, where the requirement's argument is the context itself.
//!
//! The provider trait requires `__Context__: CanCompareWith<__Context__>`, and the provider declares
//! nothing, so rustc reports `E0277` on that bound. The tool reshapes it into `[CGP-E036]` and
//! writes the context as `Self`, the name the provider uses, in the header and in the
//! `#[uses(CanCompareWith<Self>)]` help, so the generated `__Context__` never shows.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E036

use cgp::prelude::*;

pub trait CanCompareWith<Other: ?Sized> {
    fn compare_with(&self, other: &Other) -> bool;
}

impl<T: PartialEq> CanCompareWith<T> for T {
    fn compare_with(&self, other: &T) -> bool {
        self == other
    }
}

#[cgp_component(Checker)]
#[extend(CanCompareWith<Self>)]
pub trait CanCheck {
    fn check(&self) -> bool;
}

#[cgp_impl(new CheckSelf)]
impl Checker {
    fn check(&self) -> bool {
        true
    }
}

fn main() {}
