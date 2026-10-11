//! An `#[async_trait]` method with a default body.
//!
//! `E0271` on `impl Future<Output = u32> == {integer}` at the attribute; the reference quotes an
//! `E0277`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

#[async_trait]
pub trait CanFetch {
    async fn fetch(&self) -> u32 {
        1
    }
}

fn main() {}
