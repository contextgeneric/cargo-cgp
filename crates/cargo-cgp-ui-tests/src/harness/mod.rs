//! Building the tool and compiling a fixture through it.
//!
//! A fixture is a loose `.rs` file, so it is compiled by copying it into a throwaway
//! crate that depends on cgp and running `cargo-cgp check` there. Each concurrent worker
//! owns one such crate ([`ensure_worker_crates`]); reusing a worker's crate across the
//! fixtures it checks keeps cgp built and cached within that worker.

mod build;
mod cargo;
mod fixture_run;
mod manifest;
mod worker;

pub use build::*;
pub use cargo::*;
pub use fixture_run::*;
pub use manifest::*;
pub use worker::*;
