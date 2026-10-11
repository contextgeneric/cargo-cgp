//! Calling a `#[cgp_getter]` method on a context that does not wire the getter.
//!
//! The tool reports `[CGP-E001]` for `HasName` on `Person` over a `[CGP-E107]` leaf naming the
//! missing `NameGetterComponent` entry. rustc's notes point at the `#[cgp_getter]` attribute, under
//! which the generated `NameGetterComponent` marker is defined; the use-site anchor skips a
//! component marker as a context candidate, so it reads the context from `Person`'s definition.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
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
