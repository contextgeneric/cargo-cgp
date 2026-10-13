#![feature(prelude_import)]
//! Clean compile: an `#[async_trait]` method with a default body.
//!
//! The attribute rewrites the method to return `impl Future` and wraps the body in `async move`, so
//! the body becomes the returned future, as the expansion shows.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanFetch {
    fn fetch(&self) -> impl ::core::future::Future<Output = u32> {
        async move { 1 }
    }
}
fn main() {}
