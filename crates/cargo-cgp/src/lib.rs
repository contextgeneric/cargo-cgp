//! The `cargo-cgp` cargo-subcommand front-end.
//!
//! This crate contains the logic behind the `cargo-cgp` executable. Its job is small
//! and self-contained: recognize its subcommands and run cargo with the
//! [`cargo-cgp-driver`](../cargo_cgp_driver/index.html) wired in as the rustc wrapper,
//! so every workspace crate is compiled through our `rustc_driver`-based driver instead
//! of plain `rustc`. It mirrors how `cargo-clippy` launches `clippy-driver`.
//!
//! The entrypoint is [`run`], invoked by the thin `bin/cargo-cgp.rs` wrapper. Everything
//! else is library code so it stays unit-testable and free of `process::exit`.

pub mod args;
pub mod check;
pub mod config;
pub mod expand;
pub mod help;
pub mod launch;
pub mod run;
pub mod setup;
pub mod toolchain;
pub mod update;
