//! A `transform_map_fields` call whose transform and marker cannot be inferred.
//!
//! rustc's fully qualified suggestion spells `cgp::cgp_core::cgp_field::traits::TransformMapFields`
//! and `cgp::prelude::IsPresent` under the tool, where plain `cargo check` suggests the short names.
//!
//! CGP error class: none; a driver path-printing defect affecting every untransformed diagnostic.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::core::field::traits::TransformMapFields;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Point {
    pub x: u64,
}

fn main() {
    let _ = Point { x: 1 }.into_builder().transform_map_fields();
}
