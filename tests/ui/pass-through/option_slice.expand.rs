#![feature(prelude_import)]
//! A getter typed `Option<&[T]>`, which `#[cgp_auto_getter]` rejects at expansion.
//!
//! The `Option<&T>` form would read an `Option<[u8]>` field, which cannot exist because a slice is
//! unsized, so the shared field-type parser rejects the method with a message naming the sized
//! container to read instead, `Option<&Vec<T>>` from an `Option<Vec<T>>` field. The same parser
//! serves `#[cgp_getter]` and `#[implicit]` arguments. The tool must print the macro's message as
//! plain `cargo check` prints it.
//!
//! CGP error class: none; a macro's own rejection, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_auto_getter.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
fn main() {}
