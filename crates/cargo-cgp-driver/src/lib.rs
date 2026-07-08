//! The `cargo-cgp-driver` rustc wrapper.
//!
//! This crate is the `rustc_driver`-based compiler wrapper that `cargo-cgp` sets as the
//! `RUSTC_WORKSPACE_WRAPPER`, so cargo invokes it in place of `rustc` for every workspace
//! crate. It mirrors `clippy-driver`: cargo calls it as
//! `cargo-cgp-driver <path-to-rustc> <rustc args...>`, and we run the real compiler
//! in-process through [`rustc_driver`] with our own [`callbacks::CgpCallbacks`].
//!
//! For now the callbacks are a no-op, so the driver compiles exactly as `rustc` would —
//! `cargo-cgp check` is behaviourally `cargo check`. The point of routing through
//! `rustc_driver` is the hook it establishes: future work will use the callbacks to
//! inspect the compiler's diagnostics and re-present CGP errors more readably.
//!
//! # `rustc_private`
//!
//! The compiler's internal libraries are unstable, so linking them requires the
//! `rustc_private` feature and a nightly toolchain with the `rustc-dev` component. The
//! `extern crate` declarations below pull those libraries from the sysroot; add another
//! `extern crate rustc_*;` line here when a module needs a further compiler crate.
//!
//! The entrypoint is [`run::run`], invoked by the thin `bin/cargo-cgp-driver.rs` wrapper.

#![feature(rustc_private)]

extern crate rustc_driver;

pub mod args;
pub mod callbacks;
pub mod config;
pub mod run;
