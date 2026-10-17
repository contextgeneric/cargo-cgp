//! A `#[cgp_auto_dispatch]` trait method taking a `Cow<str>` with its lifetime elided.
//!
//! `E0106` suggests the lifetime the macro generated, a reserved name the program cannot write; the
//! `E0726` beside it suggests the right `'_`. The macro cannot name a lifetime a path hides, since
//! only the type's definition shows it, so the compiler reports the elision; the tool drops the
//! wrong help. This header must not spell the generated name, or the source would write it.
//!
//! CGP error class: none yet; an acceptable failure of what a macro lowers, outside the catalog.

//@quality: acceptable
//@codes: none

use std::borrow::Cow;

use cgp::prelude::*;

#[cgp_auto_dispatch]
pub trait CanLabel {
    fn label(&self, prefix: Cow<str>) -> String;
}

fn main() {}
