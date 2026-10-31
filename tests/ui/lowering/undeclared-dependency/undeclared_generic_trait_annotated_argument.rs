//! A `#[cgp_fn]` body that passes an annotated `let` to an undeclared generic trait.
//!
//! `describe(&self)` calls `self.format_pair(value)` without declaring `CanFormatPair<T>`. The
//! argument is a variable whose `let` annotation gives its type, so the `[CGP-E030]` help names
//! `#[uses(CanFormatPair<u32>)]`, as it does for a parameter's declared type.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E030

use cgp::prelude::*;

#[cgp_component(PairFormatter)]
pub trait CanFormatPair<T> {
    fn format_pair(&self, value: T) -> String;
}

#[cgp_fn]
pub fn describe(&self) -> String {
    let value: u32 = 1;
    self.format_pair(value)
}

fn main() {}
