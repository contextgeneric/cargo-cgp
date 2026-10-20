//! A CGP crate whose own attribute macro moves the tokens the user wrote into its expansion.
//!
//! `#[respanned_tokens]` (from an auxiliary crate that knows nothing of CGP) re-emits `nickname`
//! with each token keeping its own location, so the body's `holder.name.as_mut()`, an
//! `Option<&mut String>` where `Option<&String>` is declared, makes rustc suggest `.as_deref()` at
//! the end of that expression. The suggestion's span comes from an attribute macro's expansion,
//! but it edits the user's own code, not the point right after the attribute, so the tool keeps it
//! and the error matches plain `cargo check`.
//!
//! CGP error class: none; plain Rust the tool passes through.

//@aux-build: cgp-test-plain-macros

use cgp::prelude::*;
use cgp_test_plain_macros::respanned_tokens;

pub struct Holder {
    pub name: Option<String>,
}

#[respanned_tokens]
pub fn nickname(holder: &mut Holder) -> Option<&String> {
    holder.name.as_mut()
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

fn main() {}
