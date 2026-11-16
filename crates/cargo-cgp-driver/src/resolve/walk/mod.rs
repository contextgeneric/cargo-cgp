//! Walking the wiring's dependency graph down to each terminal root cause: `leaves` drives the
//! recursion and `vocabulary` decides which obligations it walks into, with the compiler queries in
//! the [query layer](crate::resolve::query).
//!
//! See `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution/walk.md`.

mod leaves;
mod vocabulary;

pub use leaves::*;
pub use vocabulary::*;
