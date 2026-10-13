#![feature(prelude_import)]
//! A `#[cgp_computer]` function whose type parameter `T` appears only in its return type, which the
//! macro rejects at expansion.
//!
//! The generated `Computer` impl's trait arguments carry only the input types, so nothing would
//! determine `T`; the macro says so on the `T` the user wrote. The tool must print the macro's
//! message as plain `cargo check` prints it.
//!
//! CGP error class: none; a macro's own rejection, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_computer.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
fn main() {}
