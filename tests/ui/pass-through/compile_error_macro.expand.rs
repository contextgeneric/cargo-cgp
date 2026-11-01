#![feature(prelude_import)]
//! `compile_error!` reports its error during expansion and lets expansion finish.
//!
//! `cargo cgp expand` prints what expanded and still fails with cargo's code.
//!
//! CGP error class: none; plain Rust the tool passes through.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub struct Kept;
fn main() {}
