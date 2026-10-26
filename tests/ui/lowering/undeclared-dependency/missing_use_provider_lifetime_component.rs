//! A higher-order provider of a component with a lifetime parameter, `CanGetName<'a>`, calls its
//! inner provider without `#[use_provider]`.
//!
//! The tool reports `[CGP-E031]` with the help `#[use_provider(Inner: NameGetter)]`, which does not
//! compile: the bound needs the lifetime, `NameGetter<'a>`, and fails with `E0106` without it.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: usability
//@codes: CGP-E031

use cgp::prelude::*;

#[cgp_component(NameGetter)]
pub trait CanGetName<'a> {
    fn name(&self, prefix: &'a str) -> &'a str;
}

#[cgp_impl(new TrimmedName<Inner>)]
impl<'a, Inner> NameGetter<'a> {
    fn name(&self, prefix: &'a str) -> &'a str {
        Inner::name(self, prefix).trim()
    }
}

fn main() {}
