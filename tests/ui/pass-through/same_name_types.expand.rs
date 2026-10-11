#![feature(prelude_import)]
//! Two distinct types with the same name, which only their module paths tell apart.
//!
//! Plain `cargo check` reports "expected `a::Foo`, found `b::Foo`" and notes that the two have
//! similar names but are distinct types. The tool must print the same; its module strip rewrites the
//! paths away, leaving "expected `Foo`, found `Foo`".
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
mod a {
    pub struct Foo;
}
mod b {
    pub struct Foo;
}
fn same_name() -> a::Foo {
    b::Foo
}
fn main() {
    let _ = same_name();
}
