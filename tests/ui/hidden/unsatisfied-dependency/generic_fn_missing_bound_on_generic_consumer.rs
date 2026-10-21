//! A method of a consumer trait with a parameter, called on an unbounded type parameter.
//!
//! `fn run<Context>` calls `context.show(1u32)` without bounding `Context` by `CanShow<T>`, whose
//! parameter the bound would have to name. The call writes `u32`, but a bound written from a call's
//! arguments would be a guess, so the tool names no bound and keeps rustc's error.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Shower)]
pub trait CanShow<T> {
    fn show(&self, value: T);
}

fn run<Context>(context: &Context) {
    context.show(1u32);
}

fn main() {}
