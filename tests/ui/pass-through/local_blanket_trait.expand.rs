#![feature(prelude_import)]
//! An ordinary blanket trait in a crate that uses `cgp`, with its bound unmet.
//!
//! `Describe` is implemented for every `T: Display`, and `App` is not `Display`. Nothing CGP is
//! involved, so the tool must print what plain `cargo check` prints; it reads any local blanket
//! trait as CGP and reports `[CGP-E009]` over a `[CGP-E201]` chain that calls `App` a context.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
#![allow(unused_imports)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::fmt::Display;
use cgp::prelude::*;
pub trait Describe {
    fn describe(&self) -> String;
}
impl<T: Display> Describe for T {
    fn describe(&self) -> String {
        ::alloc::__export::must_use({ ::alloc::fmt::format(format_args!("{0}", self)) })
    }
}
pub struct App;
fn show<T: Describe>(value: &T) -> String {
    value.describe()
}
fn main() {
    let app = App;
    let _ = show(&app);
    let _ = app.describe();
}
