//! Building and running the wrapped `cargo check`.

use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use anyhow::Context;

use crate::check::{driver_path, sysroot};
use crate::config::{DRIVER_BIN, SYSROOT_ENV};

/// Name of the OS variable that lists directories searched for dynamic libraries. The
/// driver links `librustc_driver` from the sysroot, so that directory must be on this
/// path when cargo spawns the driver.
#[cfg(target_os = "macos")]
const DYLIB_PATH_VAR: &str = "DYLD_FALLBACK_LIBRARY_PATH";
#[cfg(target_os = "windows")]
const DYLIB_PATH_VAR: &str = "PATH";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const DYLIB_PATH_VAR: &str = "LD_LIBRARY_PATH";

/// Run `cargo check` with the cargo-cgp driver wired in as the workspace rustc wrapper,
/// and return the exit code so the caller can propagate it.
///
/// `forwarded_args` are the arguments after `check`; they pass straight through to `cargo
/// check`. The front-end does nothing to the diagnostics: the driver's emitter applies
/// every CGP transform in-process and renders the result (text or JSON, matching whatever
/// format the invocation asks for), so the front-end only wires the driver in and lets
/// cargo's output stream to the terminal untouched. Because nothing is captured, cargo's
/// progress and diagnostics appear live, exactly as a plain `cargo check` would.
pub fn run_check(forwarded_args: &[String]) -> anyhow::Result<i32> {
    // The compiler cargo would otherwise use — the one whose sysroot the driver needs.
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());

    let driver = driver_path(DRIVER_BIN)?;
    let sysroot = sysroot(&rustc)?;

    let mut command = Command::new("cargo");
    command.arg("check").args(forwarded_args);

    // Route only workspace crates through the driver (dependencies keep using plain
    // rustc), exactly as `cargo clippy` does with `clippy-driver`.
    command.env("RUSTC_WORKSPACE_WRAPPER", &driver);
    command.env(SYSROOT_ENV, &sysroot);
    prepend_dylib_path(&mut command, &sysroot.join("lib"));

    // Let cargo inherit our stdio so its output streams straight through untouched.
    let status = command
        .status()
        .context("failed to run `cargo check` (is cargo on PATH?)")?;

    Ok(status.code().unwrap_or(1))
}

/// Prepend `dir` to the dynamic-library search path of `command`, preserving any
/// existing value, so the spawned driver can load `librustc_driver` from the sysroot.
fn prepend_dylib_path(command: &mut Command, dir: &Path) {
    let mut entries = vec![OsString::from(dir)];
    if let Some(existing) = env::var_os(DYLIB_PATH_VAR) {
        entries.extend(env::split_paths(&existing).map(OsString::from));
    }

    let joined = env::join_paths(entries).expect("sysroot lib path contains an invalid character");
    command.env(DYLIB_PATH_VAR, joined);
}
