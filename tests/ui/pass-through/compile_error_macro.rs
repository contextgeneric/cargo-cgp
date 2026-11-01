//! `compile_error!` reports its error during expansion and lets expansion finish.
//!
//! `cargo cgp expand` prints what expanded and still fails with cargo's code.
//!
//! CGP error class: none; plain Rust the tool passes through.

compile_error!("this crate refuses to compile");

pub struct Kept;

fn main() {}
