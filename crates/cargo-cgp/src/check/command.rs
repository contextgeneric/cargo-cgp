//! Building and running the wrapped `cargo check`.

use std::env;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;

use crate::check::dylib::prepend_dylib_path;
use crate::check::{driver_path, preflight, sysroot};
use crate::config::{
    CHECK_TARGET_DIR, DRIVER_BIN, NO_MANAGE_ENV, RUSTUP_TOOLCHAIN_ENV, SYSROOT_ENV,
};
use crate::toolchain::pinned_toolchain;

/// Run `cargo check` with the cargo-cgp driver wired in as the workspace rustc wrapper,
/// and return the exit code so the caller can propagate it.
///
/// `forwarded_args` are the arguments after `check`; they pass straight through to `cargo
/// check`. The front-end does nothing to the diagnostics: the driver's emitter applies
/// every CGP transform in-process and renders the result (text or JSON, matching whatever
/// format the invocation asks for), so the front-end only wires the driver in and lets
/// cargo's output stream to the terminal untouched.
///
/// Unless [`NO_MANAGE_ENV`] is set, the tool *manages* the toolchain: it runs the
/// [`preflight`] to confirm a matching driver and the pinned toolchain are present, and
/// forces `RUSTUP_TOOLCHAIN` to the pinned nightly so the sysroot and `librustc_driver`
/// match the compiler the driver embeds — independent of the project's own toolchain. When
/// the variable is set (local development), it skips both and trusts the environment.
pub fn run_check(forwarded_args: &[String]) -> anyhow::Result<i32> {
    let managed = env::var_os(NO_MANAGE_ENV).is_none();
    let driver = driver_path(DRIVER_BIN)?;

    // Resolve the sysroot and, when managing, the toolchain to force. The preflight
    // discovers the pinned sysroot as it verifies the driver, so we reuse it.
    let (sysroot, toolchain) = if managed {
        let toolchain = pinned_toolchain();
        let sysroot = preflight::run(&driver, &toolchain)?;
        (sysroot, Some(toolchain))
    } else {
        let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
        (sysroot(&rustc, None)?, None)
    };

    let mut command = Command::new("cargo");
    command.arg("check").args(forwarded_args);

    // Route only workspace crates through the driver (dependencies keep using plain
    // rustc), exactly as `cargo clippy` does with `clippy-driver`.
    command.env("RUSTC_WORKSPACE_WRAPPER", &driver);
    command.env(SYSROOT_ENV, &sysroot);
    prepend_dylib_path(&mut command, &sysroot.join("lib"));

    // Force the pinned nightly so the whole check compiles under the compiler the driver
    // embeds, whatever the project pins. Skipped in unmanaged mode.
    if let Some(toolchain) = &toolchain {
        command.env(RUSTUP_TOOLCHAIN_ENV, toolchain);
    }

    // Build into an isolated target directory so the check never contends with the
    // project's own builds (or Rust Analyzer's), unless the caller chose a directory.
    inject_target_dir(&mut command, forwarded_args);

    // Let cargo inherit our stdio so its output streams straight through untouched.
    let status = command
        .status()
        .context("failed to run `cargo check` (is cargo on PATH?)")?;

    Ok(status.code().unwrap_or(1))
}

/// Add `--target-dir target/cgp` unless the caller already chose the target directory —
/// via a forwarded `--target-dir` (in either `--target-dir X` or `--target-dir=X` form) or
/// the `CARGO_TARGET_DIR` environment variable, both of which take precedence.
fn inject_target_dir(command: &mut Command, forwarded_args: &[String]) {
    if env::var_os("CARGO_TARGET_DIR").is_some() || forwards_target_dir(forwarded_args) {
        return;
    }
    command
        .arg("--target-dir")
        .arg(PathBuf::from(CHECK_TARGET_DIR));
}

/// Whether the forwarded arguments already set `--target-dir` (in either `--target-dir X`
/// or `--target-dir=X` form), in which case the default is not injected.
pub fn forwards_target_dir(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--target-dir" || arg.starts_with("--target-dir="))
}
