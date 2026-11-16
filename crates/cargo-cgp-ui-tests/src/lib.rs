//! A custom UI-test harness for `cargo-cgp`, modeled on Clippy's `compile-test`, driven by the
//! `harness = false` test in [`tests/ui.rs`](../../tests/ui.rs), which calls [`run`]: each fixture
//! under `tests/ui/` runs through the [`passes`], on a pool of workers (see [`runner`]). See
//! `cgp-knowledge-base/cargo-cgp/implementation/testing.md`.

// Every public item carries a doc line, which the crate's clippy and doc runs hold.
#![warn(missing_docs)]

pub mod aux;
pub mod blocks;
pub mod cgp_code;
pub mod directives;
pub mod fixtures;
pub mod harness;
pub mod normalize;
pub mod options;
pub mod passes;
pub mod paths;
pub mod runner;
pub mod snapshot;
pub mod suite;

pub use suite::run;
