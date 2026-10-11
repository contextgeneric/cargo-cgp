//! A `transform_map_fields` call whose transform and marker cannot be inferred.
//!
//! The error is plain Rust: type annotations are needed. The tool must print what plain
//! `cargo check` prints, including the short paths in rustc's fully qualified suggestion.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use cgp::core::field::traits::TransformMapFields;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Point {
    pub x: u64,
}

fn main() {
    let _ = Point { x: 1 }.into_builder().transform_map_fields();
}
