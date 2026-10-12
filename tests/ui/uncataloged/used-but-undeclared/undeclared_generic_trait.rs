//! A `#[cgp_fn]` body that calls a method of a *generic* consumer trait without declaring it.
//!
//! `describe` calls `self.format_pair((1_u32, 2_u64))`, a method of `CanFormatPair<T>`, but its
//! `#[uses]` list is empty, so the generated context is not bounded by the trait and the call
//! cannot resolve. The trait takes a parameter, so the dependency to declare is the instantiation
//! the call's written argument fixes, and the `[CGP-E030]` help names
//! `#[uses(CanFormatPair<(u32, u64)>)]`; `#[uses(CanFormatPair)]` alone would not compile.
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
pub fn describe(&self) -> String {
    self.format_pair((1_u32, 2_u64))
}

fn main() {}
