//! `#[default_impl]` on a provider written in the explicit-context form, `impl<Context> … for Context`.
//!
//! The registration copies the `Context` parameter, which nothing in it uses, so the compiler
//! rejects it with `E0207` on `Context`; `[CGP-E018]` points at the concise form instead.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).

//@quality: acceptable
//@codes: CGP-E018

use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
pub trait CanShow<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl<Context> ShowImpl<String> for Context {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}

fn main() {}
