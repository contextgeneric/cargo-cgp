//! A variant helper attribute of another derive on a `#[derive(CgpData)]` enum.
//!
//! `#[default]` belongs to `#[derive(Default)]`, which the enum has, but the extractor's companion
//! enums copy the variant attribute without the derive. The output says the attribute cannot be found
//! and twice suggests adding the `#[derive(Default)]` that is already there.
//!
//! CGP error class: none; a `cgp` derive defect recorded in the extractor's Known issues.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[derive(Default, CgpData)]
pub enum Mode {
    #[default]
    Off,
    On(u64),
}

fn main() {}
