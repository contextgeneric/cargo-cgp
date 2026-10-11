#![feature(prelude_import)]
//! An `#[async_trait]` method with a default body.
//!
//! `E0271` on `impl Future<Output = u32> == {integer}` at the attribute; the reference quotes an
//! `E0277`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanFetch {
    fn fetch(&self) -> impl ::core::future::Future<Output = u32> {
        1
    }
}
fn main() {}
