//! An unclosed delimiter ends parsing, so nothing is expanded.
//!
//! The parser cannot recover, so `cargo cgp expand` has nothing to print, its `.expand.rs` holds the
//! marker, and the command fails with cargo's code.
//!
//! CGP error class: none; plain Rust the tool passes through.

fn main() {
