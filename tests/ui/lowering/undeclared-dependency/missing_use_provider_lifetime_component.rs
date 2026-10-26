//! A higher-order provider of a component with a lifetime parameter, `CanGetName<'a>`, calls its
//! inner provider without `#[use_provider]`.
//!
//! The tool reports `[CGP-E031]` with no help: the bound needs the lifetime (`NameGetter<'a>`), and
//! the help, which writes the component's type parameters from the call, cannot write it, so a bare
//! `#[use_provider(Inner: NameGetter)]`, which fails with `E0106`, is not offered.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
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
