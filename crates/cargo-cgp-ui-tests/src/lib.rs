//! A custom UI-test harness for `cargo-cgp`, modeled on Clippy's `compile-test`.
//!
//! Each fixture under `tests/ui/` is checked through three passes (see [`passes`]): one runs the
//! whole tool end to end (front-end and driver) and pins its rendered `.cgp.stderr`, so that
//! snapshot changes when the tool reformats diagnostics; one runs plain `cargo check` to record the
//! untransformed `.rust.stderr` baseline the tool improves on; and one runs `cargo cgp expand` to
//! pin the `.expand.rs` code the fixture's CGP macros generate. Two checks compare the passes with
//! each other (the exit status, and for `pass-through/` fixtures the output itself), and a
//! fixture's `//@json` directive adds a JSON pass (see [`directives`]). It is driven by the
//! `harness = false` test in [`tests/ui.rs`](../../tests/ui.rs), which calls [`run`]; the logic
//! lives in this library so it stays small and out of the `bin`/test entrypoint. Fixtures are
//! checked in parallel across a pool of workers (see [`runner`]). See
//! cgp-knowledge-base/cargo-cgp/implementation/testing.md.

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
