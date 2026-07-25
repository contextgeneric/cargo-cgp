//! Verifying, before a check, that a matching driver and toolchain are present.
//!
//! `cargo cgp check` never provisions anything itself; it runs this read-only preflight
//! and, on any failure, stops with a message pointing at `cargo cgp setup`. The checks run
//! in order — is the pinned toolchain installed, and does the driver run and match — so a
//! failure of the driver can be told apart from a missing toolchain underneath it. The
//! side-effecting [`run`] gathers the facts; the pure [`evaluate`] decides on them, so the
//! verdict is unit-tested without spawning anything.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, anyhow, bail};

use crate::config::{RUSTUP_TOOLCHAIN_ENV, SYSROOT_ENV, TOOL_VERSION};
use crate::launch::dylib::prepend_dylib_path;
use crate::launch::sysroot::sysroot;
use crate::toolchain::rustc_version;

/// The identity a `cargo-cgp-driver --version` reports, parsed from its output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverVersion {
    /// The driver crate's version — must equal the front-end's exactly.
    pub tool_version: String,
    /// The nightly channel the driver is pinned to.
    pub pinned_toolchain: String,
    /// The `rustc --version` line of the compiler that built the driver — must equal the
    /// installed pinned toolchain's `rustc --version`.
    pub built_against_rustc: String,
}

/// Run the preflight and, on success, return the discovered pinned sysroot (so the caller
/// reuses it rather than discovering it again). `driver` is the resolved driver path and
/// `toolchain` the effective pinned toolchain. Any failure is an error whose message names
/// the problem and directs the user to `cargo cgp setup`.
pub fn run(driver: &Path, toolchain: &str) -> anyhow::Result<PathBuf> {
    // Is the pinned toolchain installed? Doing this first tells a missing toolchain apart
    // from a bad driver, and yields the compiler identity to compare the driver against.
    let installed_rustc = rustc_version(toolchain).map_err(|e| {
        anyhow!("{e}\n\nThe pinned toolchain is not installed. Run `cargo cgp setup`.")
    })?;

    // The sysroot the check will force, also used as the driver's dylib search path below.
    let sysroot = sysroot("rustc", Some(toolchain))?;

    // Does the driver run and match? Running `--version` under the check's own environment
    // is both a load test (a driver built against another nightly cannot find its
    // `librustc_driver` here and fails to launch) and a version handshake.
    let mut command = Command::new(driver);
    command.env(RUSTUP_TOOLCHAIN_ENV, toolchain);
    command.env(SYSROOT_ENV, &sysroot);
    prepend_dylib_path(&mut command, &sysroot.join("lib"));
    command.arg("--version");

    let output = command.output().map_err(|e| {
        anyhow!(
            "failed to run the cargo-cgp-driver at {}: {e}\n\nRun `cargo cgp setup`.",
            driver.display()
        )
    })?;

    if !output.status.success() {
        bail!(
            "the cargo-cgp-driver could not run under toolchain `{toolchain}` \
             (it was likely built against a different nightly). Run `cargo cgp setup`."
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .context("`cargo-cgp-driver --version` produced non-UTF-8 output")?;
    let reported = parse_driver_version(&stdout).ok_or_else(|| {
        anyhow!("could not parse `cargo-cgp-driver --version` output. Run `cargo cgp setup`.")
    })?;

    evaluate(TOOL_VERSION, &reported, &installed_rustc)
        .map_err(|reason| anyhow!("{reason}\n\nRun `cargo cgp setup`."))?;

    Ok(sysroot)
}

/// Parse the driver's `--version` output into a [`DriverVersion`]. Returns `None` if the
/// shape is not what [`crate`]'s driver prints (an old or foreign binary).
pub fn parse_driver_version(output: &str) -> Option<DriverVersion> {
    let mut lines = output.lines();

    let tool_version = lines
        .next()?
        .strip_prefix("cargo-cgp-driver")?
        .trim()
        .to_owned();
    if tool_version.is_empty() {
        return None;
    }

    let mut pinned_toolchain = None;
    let mut built_against_rustc = None;
    for line in lines {
        if let Some(value) = line.strip_prefix("pinned-toolchain:") {
            pinned_toolchain = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("built-against-rustc:") {
            built_against_rustc = Some(value.trim().to_owned());
        }
    }

    Some(DriverVersion {
        tool_version,
        pinned_toolchain: pinned_toolchain?,
        built_against_rustc: built_against_rustc?,
    })
}

/// Decide whether the reported driver matches this front-end and the installed toolchain,
/// with strict equality on both the version and the built-against compiler. Pure, so the
/// pass/fail decision and its message are tested without a driver or `rustc`.
pub fn evaluate(
    frontend_version: &str,
    driver: &DriverVersion,
    installed_rustc: &str,
) -> Result<(), String> {
    if driver.tool_version != frontend_version {
        return Err(format!(
            "the installed cargo-cgp-driver is version {}, but this cargo-cgp is {frontend_version} \
             (the two are out of lockstep)",
            driver.tool_version
        ));
    }

    if driver.built_against_rustc != installed_rustc {
        return Err(format!(
            "the cargo-cgp-driver was built against `{}`, but the pinned toolchain \
             `{}` now provides `{installed_rustc}`",
            driver.built_against_rustc, driver.pinned_toolchain
        ));
    }

    Ok(())
}
