//! A struct deriving `HasField` for a field its `Deref` target also has.
//!
//! The `E0119` overlap with `cgp`'s `Deref` blanket impl keeps its headline but loses rustc's note
//! naming the blanket impl, and gains a `help` to use `#[derive(HasField)]` over a `[CGP-E108]` root
//! cause saying the impl is missing. The impl exists twice; every added line is false.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E105 CGP-E108

use core::ops::Deref;

use cgp::prelude::*;

#[derive(HasField)]
pub struct Inner {
    pub name: String,
}

#[derive(HasField)]
pub struct Outer {
    pub name: String,
    pub inner: Inner,
}

impl Deref for Outer {
    type Target = Inner;

    fn deref(&self) -> &Inner {
        &self.inner
    }
}

fn main() {}
