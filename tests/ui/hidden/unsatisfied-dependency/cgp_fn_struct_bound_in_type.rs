//! A `#[cgp_fn]` trait required by a struct's bound, where the struct is only named in a type, so no
//! expression carries the context. The root cause is the missing `name` field on `App`.
//!
//! The anchors read a context from an expression or from a struct definition, and neither names
//! `App` here, so rustc's diagnostic stays, with its `HasField` clause reworded. It states the cause
//! truly but leads with the bound.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_fn]
fn get_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

pub struct Holder<T: GetName>(pub T);

#[derive(HasField)]
pub struct App {
    pub label: String,
}

pub struct Outer {
    pub holder: Holder<App>,
}

fn main() {}
