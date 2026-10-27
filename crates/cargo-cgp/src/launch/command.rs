//! Building the wrapped cargo command both diagnostic subcommands run.

use std::env;
use std::process::Command;

use crate::config::{DRIVER_BIN, NO_MANAGE_ENV, RUSTUP_TOOLCHAIN_ENV, SYSROOT_ENV};
use crate::launch::dylib::prepend_dylib_path;
use crate::launch::invocation::cargo_invocation;
use crate::launch::target_dir::default_target_dir;
use crate::launch::workspace_wrapper::{WORKSPACE_WRAPPER_ENV, workspace_wrapper_warning};
use crate::launch::{driver_path, preflight, sysroot};
use crate::toolchain::pinned_toolchain;

/// Build a `cargo <subcommand>` command with the cargo-cgp driver wired in as the workspace
/// rustc wrapper, ready to run.
///
/// `forwarded_args` are the arguments after the cargo-cgp subcommand; they pass straight
/// through to cargo. `tool_cargo_args` and `tool_rustc_args` are what the subcommand adds for
/// cargo and for rustc (`expand`'s `--profile check` and its marker flags); [`cargo_invocation`]
/// places each on its side of a single `--`.
///
/// Unless [`NO_MANAGE_ENV`] is set, the tool *manages* the toolchain: it runs the
/// `preflight` to confirm a matching driver and the pinned toolchain are present, and
/// forces `RUSTUP_TOOLCHAIN` to the pinned nightly so the sysroot and `librustc_driver`
/// match the compiler the driver embeds — independent of the project's own toolchain. When
/// the variable is set (local development), it skips both and trusts the environment.
pub fn wrapped_cargo(
    subcommand: &str,
    forwarded_args: &[String],
    tool_cargo_args: &[String],
    tool_rustc_args: &[String],
) -> anyhow::Result<Command> {
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

    // Route only workspace crates through the driver (dependencies keep using plain
    // rustc), exactly as `cargo clippy` does with `clippy-driver`. A wrapper of the user's own in
    // the same variable cannot run as well, so the user is told.
    if let Some(warning) =
        workspace_wrapper_warning(env::var_os(WORKSPACE_WRAPPER_ENV).as_deref(), &driver.path)
    {
        eprintln!("{warning}");
    }
    command.env(WORKSPACE_WRAPPER_ENV, &driver.path);
    command.env(SYSROOT_ENV, &sysroot);
    prepend_dylib_path(&mut command, &sysroot.join("lib"))?;

    // Force the pinned nightly so the whole build runs under the compiler the driver
    // embeds, whatever the project pins. Skipped in unmanaged mode.
    if let Some(toolchain) = &toolchain {
        command.env(RUSTUP_TOOLCHAIN_ENV, toolchain);
    }

    // Build into an isolated target directory so the run never contends with the project's
    // own builds (or Rust Analyzer's), unless the caller chose a directory. This comes after the
    // toolchain is forced, because the directory query runs under the same toolchain.
    let target_dir = default_target_dir(&command, forwarded_args);
    command.args(cargo_invocation(
        subcommand,
        forwarded_args,
        tool_cargo_args,
        target_dir.as_deref(),
        tool_rustc_args,
    ));

    Ok(command)
}
