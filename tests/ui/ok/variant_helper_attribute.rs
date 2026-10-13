//! Clean compile: a variant helper attribute of another derive on a `#[derive(CgpData)]` enum.
//!
//! `#[default]` belongs to `#[derive(Default)]`, which the enum has. The extractor's companion enums
//! copy each variant but keep only its `#[cfg]` and documentation attributes, so the `#[default]`
//! stays on `Mode`, as the expansion shows.

use cgp::prelude::*;

#[derive(Default, CgpData)]
pub enum Mode {
    #[default]
    Off,
    On(u64),
}

fn main() {}
