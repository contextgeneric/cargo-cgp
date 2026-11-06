//! Two `#[default_impl]` registrations of a provider struct that is itself generic over the impl's
//! parameter, which `[CGP-E018]` cannot write the fix for: one at a path key
//! (`@app.ShowImplComponent`), whose key gives no type to put in the parameter's place, and one of
//! `ShowPair<T, U>` at `String`, which would still name `U`. Each registration copies the provider
//! impl's parameters, so each parameter neither the key nor the table uses is unconstrained (E0207).
//!
//! The tool leaves the three errors as rustc wrote them. A registration is not a wiring entry, so the
//! `[CGP-E019]` detector, tried after `[CGP-E018]`, skips an impl whose parameter a provider impl
//! also declares; the shape is an open issue of the class.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/unconstrained-generic.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use core::fmt::Display;
use core::marker::PhantomData;

use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
#[prefix(@app in AppNamespace)]
pub trait CanShow<T> {
    fn show(&self, value: &T) -> String;
}

cgp_namespace! {
    new AppNamespace
}

pub struct ShowWith<T>(pub PhantomData<T>);

#[cgp_impl(ShowWith<T>)]
#[default_impl(@app.ShowImplComponent in AppNamespace)]
impl<T: Display> ShowImpl<T> {
    fn show(&self, value: &T) -> String {
        format!("{value}")
    }
}

pub struct ShowPair<T, U>(pub PhantomData<(T, U)>);

#[cgp_impl(ShowPair<T, U>)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl<T: Display, U> ShowImpl<T> {
    fn show(&self, value: &T) -> String {
        format!("{value}")
    }
}

fn main() {}
