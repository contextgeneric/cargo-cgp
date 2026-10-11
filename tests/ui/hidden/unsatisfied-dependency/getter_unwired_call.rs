//! Calling a `#[cgp_getter]` method on a context that does not wire the getter.
//!
//! The headline names `NameGetterComponent` as the context, over a `[CGP-E107]` leaf saying the same
//! marker lacks its own entry. The context is `Person`; plain rustc names it correctly.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E107

use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

fn main() {
    let _ = Person { name: String::new() }.name();
}
