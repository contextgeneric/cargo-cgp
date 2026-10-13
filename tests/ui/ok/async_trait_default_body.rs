//! Clean compile: an `#[async_trait]` method with a default body.
//!
//! The attribute rewrites the method to return `impl Future` and wraps the body in `async move`, so
//! the body becomes the returned future, as the expansion shows.

use cgp::prelude::*;

#[async_trait]
pub trait CanFetch {
    async fn fetch(&self) -> u32 {
        1
    }
}

fn main() {}
