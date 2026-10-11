#![feature(prelude_import)]
//! A user trait whose only item is `type Delegate`, implemented twice for one type.
//!
//! The two impls conflict (`E0119`), a plain coherence error. The tool must print what plain
//! `cargo check` prints; it reads any trait with a lone `type Delegate` as a CGP namespace and
//! reports `[CGP-E004]` duplicate wiring.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
#![allow(unused_imports)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait Lookup<Table> {
    type Delegate;
}
pub struct Key;
pub struct Table;
impl Lookup<Table> for Key {
    type Delegate = u8;
}
impl Lookup<Table> for Key {
    type Delegate = u16;
}
fn main() {}
