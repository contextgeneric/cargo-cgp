//! Two `#[cgp_auto_dispatch]` traits in one module that both declare a method
//! named `area`. The macro names the per-variant helper `__compute_area__` and
//! the computer `ComputeArea` after the method alone, so the module defines each
//! twice: `E0428` on both names, then `E0119` on the computers' conflicting
//! impls. This is a known defect of the macro: folding the trait name into the
//! generated names would remove the clash, at the cost of renaming the public
//! `Compute{Method}` provider. The `ComputeArea` carets fall on the second `area`
//! method name, since the computer's name is spanned on the method identifier
//! it derives from; the helper's carets cover the whole attribute.
//!
//! Error class: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md, with the name clash itself
//! under https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/error_codes/e0428.md.

use cgp::prelude::*;

#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}

#[cgp_auto_dispatch]
pub trait HasScaledArea {
    fn area(&self) -> f64;
}

fn main() {}
