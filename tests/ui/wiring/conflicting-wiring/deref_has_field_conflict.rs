//! A struct deriving `HasField` for a field its `Deref` target also has.
//!
//! The derive's `HasField` impl overlaps `cgp`'s `Deref` blanket impl (`E0119`). The tool keeps
//! rustc's diagnostic, with the field tag resugared in the headline and rustc's note naming the
//! conflicting blanket impl. The resolver takes only an unmet obligation, and an overlap is not one,
//! so it adds no missing-derive help or cause.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).

//@quality: acceptable
//@codes: none

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
