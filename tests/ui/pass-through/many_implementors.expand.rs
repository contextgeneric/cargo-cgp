#![feature(prelude_import)]
//! A plain Rust bound unmet by a type, on a trait with more than nine implementors.
//!
//! rustc's help lists the trait's implementors, eight of them and "and 2 others" in a normal
//! session. The driver injects `--verbose`, which lifts that limit, so the tool cuts the list back
//! to rustc's length on every diagnostic; the output must equal plain `cargo check`'s.
//!
//! CGP error class: none; plain Rust the tool passes through.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub trait Shape {}
impl Shape for u8 {}
impl Shape for u16 {}
impl Shape for u32 {}
impl Shape for u64 {}
impl Shape for u128 {}
impl Shape for i8 {}
impl Shape for i16 {}
impl Shape for i32 {}
impl Shape for i64 {}
impl Shape for i128 {}
fn area(_shape: impl Shape) {}
fn main() {
    area("not a shape");
}
