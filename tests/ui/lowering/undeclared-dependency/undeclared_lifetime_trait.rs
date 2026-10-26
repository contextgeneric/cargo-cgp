//! A `#[cgp_fn]` body calls a method of a consumer trait with a lifetime parameter,
//! `CanGetPrefix<'a>`, without `#[uses]`.
//!
//! The tool reports `[CGP-E030]` with no help: the help writes only type arguments, and the bound
//! needs the lifetime.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E030

use cgp::prelude::*;

#[cgp_component(PrefixGetter)]
pub trait CanGetPrefix<'a> {
    fn prefix(&self, text: &'a str) -> &'a str;
}

#[cgp_fn]
pub fn short_prefix<'a>(&self, text: &'a str) -> &'a str {
    self.prefix(text)
}

fn main() {}
