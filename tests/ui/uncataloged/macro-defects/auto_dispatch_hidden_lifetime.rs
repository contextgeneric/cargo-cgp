//! A `#[cgp_auto_dispatch]` trait method taking a `Cow<str>` with its lifetime elided.
//!
//! `E0106` suggests the generated lifetime `'__a__`; the `E0726` beside it suggests the right `'_`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use std::borrow::Cow;

use cgp::prelude::*;

#[cgp_auto_dispatch]
pub trait CanLabel {
    fn label(&self, prefix: Cow<str>) -> String;
}

fn main() {}
