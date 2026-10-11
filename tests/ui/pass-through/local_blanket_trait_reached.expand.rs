#![feature(prelude_import)]
//! An ordinary blanket trait in a crate that uses `cgp`, reached through a wrapper impl and a call.
//!
//! `Describe` is implemented for every `T: Clone`, and `App` is not `Clone`. `CheckDescribe` asserts
//! `Describe` as a supertrait, and `main` calls `app.describe()`. Nothing CGP is involved, so the
//! tool must print what plain `cargo check` prints. The impl-site and call-site anchors both accept
//! a blanket trait, but only one whose blanket depends on CGP, and `Describe`'s bound is `Clone`.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
#![allow(unused_imports)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait Describe {
    fn describe(&self) -> String;
}
impl<T: Clone> Describe for T {
    fn describe(&self) -> String {
        String::from("described")
    }
}
pub trait CheckDescribe: Describe {}
pub struct App;
impl CheckDescribe for App {}
fn main() {
    let app = App;
    let _ = app.describe();
}
