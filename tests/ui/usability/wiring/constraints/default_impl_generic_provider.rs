//! `#[default_impl]` on a provider impl generic over the dispatched type.
//!
//! `E0207` with rustc's "remove the unused type parameter" help, which is not the fix.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
