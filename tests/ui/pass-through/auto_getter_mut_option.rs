//! A `&mut self` auto getter returning `Option<&String>`, which the generated body cannot produce.
//!
//! The error is a plain type mismatch, and rustc's suggestion is spliced after the attribute (a
//! known `cgp` defect). The tool must print what plain `cargo check` prints, short paths included
//! (`Option<&mut String>`, not `std::option::Option<&mut std::string::String>`).
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

//@json

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasNickname {
    fn nickname(&mut self) -> Option<&String>;
}

fn main() {}
