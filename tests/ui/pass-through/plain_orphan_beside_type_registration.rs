//! A plain Rust orphan impl in a crate that also makes legal per-type registrations.
//!
//! An `E0117` names no machinery parameter, so the orphan stage scans the crate's registrations
//! for the impl at its caret. These two are legal, since `DefaultImpls1<ShowImplComponent>` carries
//! a local component, and neither sits at the caret, so the error must stay as rustc wrote it
//! rather than read as `[CGP-E022]` about a key the program registered correctly.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<String> {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}

#[cgp_impl(new ShowNumber)]
#[default_impl(u64 in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<u64> {
    fn show(&self, value: &u64) -> String {
        value.to_string()
    }
}

impl std::fmt::Display for Vec<u8> {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

fn main() {}
