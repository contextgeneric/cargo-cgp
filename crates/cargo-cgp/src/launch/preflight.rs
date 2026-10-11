//! Verifying, before a check, that a matching driver and toolchain are present.
//!
//! `cargo cgp check` never provisions anything itself; it runs this read-only preflight
//! and, on any failure, stops with a message pointing at `cargo cgp setup`. The checks run
//! in order — is the pinned toolchain installed, and does the driver run and match — so a
//! failure of the driver can be told apart from a missing toolchain underneath it. The
//! side-effecting [`run`] gathers the facts; the pure [`evaluate`] decides on them and
//! [`failure_message`] words each failure, so both are unit-tested without spawning anything.

use std::path::PathBuf;
use std::process::Command;

use anyhow::anyhow;

use crate::config::{RUSTUP_TOOLCHAIN_ENV, SYSROOT_ENV, TOOL_VERSION};
use crate::launch::driver_path::DriverLocation;
use crate::launch::dylib::prepend_dylib_path;
use crate::launch::sysroot::{format_stderr, sysroot};
use crate::toolchain::{ToolchainError, rustc_version};

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

/// A reason the preflight stops the check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightFailure {
    /// `rustup` is not on `PATH`, so the toolchain can be neither queried nor installed.
    RustupMissing,
    /// rustup could not run the toolchain's `rustc`, usually because it is not installed.
    ToolchainUnavailable { stderr: String },
    /// The driver process could not be started at all.
    DriverDidNotStart { error: String },
    /// The driver started but failed, usually because the loader could not find the
    /// `librustc_driver` it was built against.
    DriverDidNotRun { stderr: String },
    /// The driver's `--version` output is not the shape this front-end expects.
    UnparsableVersion,
    /// The driver is another release than the front-end.
    VersionMismatch { driver_version: String },
    /// The driver was built by another compiler than the toolchain now provides.
    RustcMismatch {
        built_against: String,
        installed: String,
    },
}

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

/// The preflight failure a failed toolchain query stands for, or the error itself when it is
/// neither a missing rustup nor a query rustup refused (an unreadable answer, say), which has no
/// `setup` to suggest.
pub fn toolchain_failure(error: ToolchainError) -> Result<PreflightFailure, anyhow::Error> {
    match error {
        ToolchainError::RustupMissing => Ok(PreflightFailure::RustupMissing),
        ToolchainError::QueryFailed { stderr } => {
            Ok(PreflightFailure::ToolchainUnavailable { stderr })
        }
        ToolchainError::Other(error) => Err(error),
    }
}

/// Judge the answer of a driver that started for `--version`: whether it ran (`succeeded`), and
/// what it printed. A failed run keeps its stderr, the loader's reason; a run whose output is not
/// the driver's three lines is unparsable; otherwise [`evaluate`] compares its identity.
pub fn judge_driver_answer(
    succeeded: bool,
    stdout: &[u8],
    stderr: &[u8],
    frontend_version: &str,
    installed_rustc: &str,
) -> Result<(), PreflightFailure> {
    if !succeeded {
        return Err(PreflightFailure::DriverDidNotRun {
            stderr: String::from_utf8_lossy(stderr).into_owned(),
        });
    }
    let reported = parse_driver_version(&String::from_utf8_lossy(stdout))
        .ok_or(PreflightFailure::UnparsableVersion)?;
    evaluate(frontend_version, &reported, installed_rustc)
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
/// pass/fail decision is tested without a driver or `rustc`.
pub fn evaluate(
    frontend_version: &str,
    driver: &DriverVersion,
    installed_rustc: &str,
) -> Result<(), PreflightFailure> {
    if driver.tool_version != frontend_version {
        return Err(PreflightFailure::VersionMismatch {
            driver_version: driver.tool_version.clone(),
        });
    }

    if driver.built_against_rustc != installed_rustc {
        return Err(PreflightFailure::RustcMismatch {
            built_against: driver.built_against_rustc.clone(),
            installed: installed_rustc.to_owned(),
        });
    }

    Ok(())
}

/// The message for a preflight `failure`. `toolchain` is the toolchain the preflight queried,
/// `driver` the driver it ran, and `frontend_version` this front-end's version.
///
/// Every message about the driver names its path and where it was found, and every failure
/// `setup` can repair ends by naming it. A missing rustup is the exception: `setup` needs rustup
/// too, so that message names the two ways forward instead.
pub fn failure_message(
    failure: &PreflightFailure,
    toolchain: &str,
    driver: &DriverLocation,
    frontend_version: &str,
) -> String {
    const SETUP: &str = "\n\nRun `cargo cgp setup`.";
    let driver = driver.describe();

    match failure {
        PreflightFailure::RustupMissing => format!(
            "rustup was not found on PATH, and cargo-cgp installs and selects its toolchain \
             `{toolchain}` through rustup\n\nInstall rustup, or install cargo-cgp through its \
             Nix flake, which needs no rustup."
        ),
        PreflightFailure::ToolchainUnavailable { stderr } => format!(
            "toolchain `{toolchain}` is not available{}{SETUP}",
            format_stderr(stderr.as_bytes())
        ),
        PreflightFailure::DriverDidNotStart { error } => {
            format!("failed to run {driver}: {error}{SETUP}")
        }
        PreflightFailure::DriverDidNotRun { stderr } => format!(
            "{driver} could not run under toolchain `{toolchain}` (it was likely built against a \
             different nightly){}{SETUP}",
            format_stderr(stderr.as_bytes())
        ),
        PreflightFailure::UnparsableVersion => {
            format!("could not parse the `--version` output of {driver}{SETUP}")
        }
        PreflightFailure::VersionMismatch { driver_version } => format!(
            "{driver} is version {driver_version}, but this cargo-cgp is {frontend_version} \
             (the two are out of lockstep){SETUP}"
        ),
        PreflightFailure::RustcMismatch {
            built_against,
            installed,
        } => format!(
            "{driver} was built against `{built_against}`, but toolchain `{toolchain}` provides \
             `{installed}`{SETUP}"
        ),
    }
}
