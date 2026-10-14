//! Upcasting from an enum that derives only `ExtractField`.
//!
//! An upcast walks the source's `HasFields` list, so the source needs `#[derive(HasFields)]` too. The
//! output is "no method named `upcast` found for enum `Small`"; `HasFields` is never mentioned.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/hidden-root-cause.md.

//@quality: hidden
//@codes: none

use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Small {
    Foo(u64),
}

#[derive(CgpData)]
pub enum Big {
    Foo(u64),
    Bar(String),
}

fn main() {
    let _ = Small::Foo(1).upcast(PhantomData::<Big>);
}
