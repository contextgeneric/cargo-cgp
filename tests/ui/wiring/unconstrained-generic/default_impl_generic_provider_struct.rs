//! `#[default_impl]` on the impl of a provider struct that is itself generic over the dispatched
//! type.
//!
//! The registration copies the impl's parameter `T` and names the provider `ShowWith<T>`, but its
//! key `String` and the table do not use `T`, so the compiler rejects it with `E0207`. The tool
//! reports `[CGP-E018]`, as for a provider struct that does not use the parameter, with a help that
//! registers the provider at the key: `#[cgp_impl(ShowWith<String>)]` with `String` in place of `T`.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).

//@quality: acceptable
//@codes: CGP-E018

use core::fmt::Display;
use core::marker::PhantomData;

use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
pub trait CanShow<T> {
    fn show(&self, value: &T) -> String;
}

pub struct ShowWith<T>(pub PhantomData<T>);

#[cgp_impl(ShowWith<T>)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl<T: Display> ShowImpl<T> {
    fn show(&self, value: &T) -> String {
        format!("{value}")
    }
}

fn main() {}
