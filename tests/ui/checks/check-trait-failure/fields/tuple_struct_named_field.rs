//! A tuple-struct context reached through an implicit argument that reads a named field.
//!
//! The resolver declines and rustc's `E0599` stays, resugared only; the named-field form of the same
//! mistake becomes `[CGP-E003]` over `[CGP-E110]`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_fn]
pub fn greet(&self, #[implicit] name: &str) -> String {
    format!("Hello, {name}")
}

#[derive(HasField)]
pub struct Person(pub String);

fn main() {
    let _ = Person(String::new()).greet();
}
