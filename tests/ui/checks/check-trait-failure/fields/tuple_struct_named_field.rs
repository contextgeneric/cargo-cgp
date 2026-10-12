//! A tuple-struct context reached through an implicit argument that reads a named field.
//!
//! The call-site anchor reads `Person` off the tuple-struct constructor and resolves the call to
//! ``[CGP-E003] missing field `name` on `Person` ``, as it does the named-field form of the same
//! mistake. No help names a fix: a tuple struct cannot gain a named field, so the help that adds
//! the field to a struct with named fields does not apply.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E003 CGP-E104 CGP-E110

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
