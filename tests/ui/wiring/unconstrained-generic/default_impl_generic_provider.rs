//! `#[default_impl]` on a provider impl generic over the dispatched type.
//!
//! The `E0207` on `T` becomes `[CGP-E018]`, with a help to write the impl for `String` alone.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
//!
//! rustc's "remove the unused type parameter" help, which is not the fix, is dropped.

//@quality: acceptable
//@codes: CGP-E018

use core::fmt::Display;

use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
pub trait CanShow<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowWithDisplay)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl<T: Display> ShowImpl<T> {
    fn show(&self, value: &T) -> String {
        format!("{value}")
    }
}

fn main() {}
