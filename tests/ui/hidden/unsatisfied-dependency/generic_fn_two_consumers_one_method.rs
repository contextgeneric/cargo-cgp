//! A method two consumer traits declare, called on an unbounded type parameter.
//!
//! `CanGreet` and `CanWelcome` both declare `greet`, and `fn run<Context>` calls `context.greet()`
//! without bounding `Context` by either. Either bound would let the call resolve, and the call does
//! not say which the reader means, so the tool names neither and keeps rustc's error, which lists
//! both traits.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_component(Welcomer)]
pub trait CanWelcome {
    fn greet(&self);
}

fn run<Context>(context: &Context) {
    context.greet();
}

fn main() {}
