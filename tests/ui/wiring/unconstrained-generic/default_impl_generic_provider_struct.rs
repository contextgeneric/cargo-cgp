//! `#[default_impl]` on the impl of a provider struct that is itself generic over the dispatched
//! type.
//!
//! The registration copies the impl's parameter `T` and names the provider `ShowWith<T>`, but its
//! key `String` and the table do not use `T`, so the compiler rejects it with `E0207`. The tool
//! passes the error through: the `#[default_impl]` reshaping covers a provider whose struct does not
//! use the parameter, and the per-entry reshaping covers entries a wiring table names.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
