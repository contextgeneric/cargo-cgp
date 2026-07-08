//! Thin entrypoint for the rustc wrapper. All logic lives in the library's
//! [`cargo_cgp_driver::run::run`]; this wrapper only returns its exit code, which
//! `Termination` turns into the process status.
//!
//! The `rustc_private` feature gate is required here as well as in the library: the
//! binary crate is what ultimately links the compiler's `rustc_driver` dylib, and that
//! link is only permitted when the crate opts into the feature.

#![feature(rustc_private)]

use std::process::ExitCode;

fn main() -> ExitCode {
    cargo_cgp_driver::run::run()
}
