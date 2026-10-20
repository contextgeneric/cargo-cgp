//! A crate without CGP whose own attribute macro writes a body of the wrong type.
//!
//! `#[generated_body]` (from an auxiliary crate that knows nothing of CGP) replaces the body of
//! `nickname` with `holder.name.as_mut()`, an `Option<&mut String>` where `Option<&String>` is
//! declared. The generated tokens carry the attribute's span, so rustc suggests `.as_deref()` right
//! after the attribute. In a crate that uses CGP the tool drops such a suggestion, since no edit of
//! the source can apply it. This crate does not use CGP, so the tool must leave the error,
//! suggestion included, exactly as rustc prints it.
//!
//! CGP error class: none; plain Rust the tool passes through.

//@aux-build: cgp-test-plain-macros

use cgp_test_plain_macros::generated_body;

pub struct Holder {
    pub name: Option<String>,
}

#[generated_body]
pub fn nickname(holder: &mut Holder) -> Option<&String> {}

fn main() {}
