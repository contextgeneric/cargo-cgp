//! A `#[cgp_fn]` body that passes one of its own parameters to an undeclared generic trait.
//!
//! `describe(&self, value: u32)` calls `self.format_pair(value)` without declaring
//! `CanFormatPair<T>`. The argument is a variable, but its type is the parameter's declared `u32`,
//! so the `[CGP-E030]` help names `#[uses(CanFormatPair<u32>)]`.
//!
//! CGP error class: a used-but-undeclared dependency, not yet in the catalog
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md#a-gap-the-used-but-undeclared-and-wrong-half-classes).

//@quality: acceptable
//@codes: CGP-E030

use cgp::prelude::*;

#[cgp_component(PairFormatter)]
pub trait CanFormatPair<T> {
    fn format_pair(&self, value: T) -> String;
}

#[cgp_fn]
pub fn describe(&self, value: u32) -> String {
    self.format_pair(value)
}

fn main() {}
