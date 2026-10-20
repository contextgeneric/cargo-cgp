//! A CGP crate calling a function-like macro whose expansion has the wrong type.
//!
//! `holder_name_mut!()` (from an auxiliary crate that knows nothing of CGP) expands to
//! `holder.name.as_mut()`, an `Option<&mut String>` where `Option<&String>` is declared, with every
//! token at the macro call. rustc suggests `.as_deref()` at the end of the call, an edit that
//! applies there. The tool drops a suggestion spliced at the end of an attribute macro alone, so it
//! keeps this one and the error matches plain `cargo check`.
//!
//! CGP error class: none; plain Rust the tool passes through.

//@aux-build: cgp-test-plain-macros

use cgp::prelude::*;
use cgp_test_plain_macros::holder_name_mut;

pub struct Holder {
    pub name: Option<String>,
}

pub fn nickname(holder: &mut Holder) -> Option<&String> {
    holder_name_mut!()
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

fn main() {}
