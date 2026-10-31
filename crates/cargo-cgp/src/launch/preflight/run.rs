//! Gathering the facts a preflight decides on.

use std::path::PathBuf;
use std::process::Command;

use anyhow::anyhow;

use crate::config::{RUSTUP_TOOLCHAIN_ENV, SYSROOT_ENV, TOOL_VERSION};
use crate::launch::driver_path::DriverLocation;
use crate::launch::dylib::prepend_dylib_path;
use crate::launch::preflight::{
    PreflightFailure, failure_message, judge_driver_answer, toolchain_failure,
};
use crate::launch::sysroot::sysroot;
use crate::toolchain::rustc_version;

/// Run the preflight and, on success, return the discovered pinned sysroot (so the caller
/// reuses it rather than discovering it again). `driver` is the located driver and `toolchain`
/// the effective pinned toolchain. Any failure is an error worded by [`failure_message`].
pub fn run(driver: &DriverLocation, toolchain: &str) -> anyhow::Result<PathBuf> {
    let fail = |failure: PreflightFailure| {
        anyhow!(failure_message(&failure, toolchain, driver, TOOL_VERSION))
    };

    // Is the pinned toolchain installed? Doing this first tells a missing toolchain apart
    // from a bad driver, and yields the compiler identity to compare the driver against.
    let installed_rustc = match rustc_version(toolchain) {
        Ok(version) => version,
        Err(error) => return Err(toolchain_failure(error).map_or_else(|error| error, fail)),
    };

    // The sysroot the check will force, also used as the driver's dylib search path below.
    let sysroot = sysroot("rustc", Some(toolchain))?;

    // Does the driver run and match? Running `--version` under the check's own environment
    // is both a load test (a driver built against another nightly cannot find its
    // `librustc_driver` here and fails to launch) and a version handshake.
    let mut command = Command::new(&driver.path);
    command.env(RUSTUP_TOOLCHAIN_ENV, toolchain);
    command.env(SYSROOT_ENV, &sysroot);
    prepend_dylib_path(&mut command, &sysroot.join("lib"))?;
    command.arg("--version");

    let output = command.output().map_err(|error| {
        fail(PreflightFailure::DriverDidNotStart {
            error: error.to_string(),
        })
    })?;

    judge_driver_answer(
        output.status.success(),
        &output.stdout,
        &output.stderr,
        TOOL_VERSION,
        &installed_rustc,
    )
    .map_err(fail)?;

    Ok(sysroot)
}
