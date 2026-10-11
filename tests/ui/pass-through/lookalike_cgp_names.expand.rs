#![feature(prelude_import)]
//! A program with its own `Symbol`, `Chars`, `Nil`, and `HasField`, unrelated to CGP's.
//!
//! The program never uses `cgp`, so its unmet `HasField` bound is plain Rust. The tool must print
//! what plain `cargo check` prints; its text pass matches the names alone, so it resugars the bound
//! to `HasField<Symbol!("a")>` and rewords the help into a `#[derive(HasField)]` hint.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::marker::PhantomData;
pub struct Chars<const C: char, T>(PhantomData<T>);
pub struct Nil;
pub struct Symbol<const N: usize, T>(PhantomData<T>);
pub trait HasField<Tag> {
    type Value;
}
pub struct Ctx;
impl HasField<Symbol!("b")> for Ctx {
    type Value = u8;
}
fn need<T: HasField<Symbol!("a")>>() {}
fn main() {
    need::<Ctx>();
}
