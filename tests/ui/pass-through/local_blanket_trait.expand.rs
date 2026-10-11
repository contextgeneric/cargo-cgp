#![feature(prelude_import)]
//! An ordinary blanket trait in a crate that uses `cgp`, with its bound unmet.
//!
//! `Describe` is implemented for every `T: Display`, and `App` is not `Display`. Nothing CGP is
//! involved, so the tool must print what plain `cargo check` prints, without re-anchoring the
//! `show(&app)` error onto the `app.describe()` call in the same function.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
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
