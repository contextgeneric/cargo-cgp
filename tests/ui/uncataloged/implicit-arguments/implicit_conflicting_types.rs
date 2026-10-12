//! Two methods of one provider reading the same implicit field as two different types.
//!
//! An `E0284` and an `E0282`, both on the provider name; nothing says the methods disagree on the
//! field's type. rustc reports the `E0284` twice, and the tool merges the repeat, which differed
//! only in a note the tool renamed, into the first block.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Counter)]
pub trait CanCount {
    fn small(&self) -> u32;
    fn large(&self) -> u64;
}

#[cgp_impl(new ReadCount)]
impl Counter {
    fn small(&self, #[implicit] count: u32) -> u32 {
        count
    }

    fn large(&self, #[implicit] count: u64) -> u64 {
        count
    }
}

fn main() {}
