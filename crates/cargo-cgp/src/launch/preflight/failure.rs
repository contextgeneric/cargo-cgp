//! Why a preflight stops the check, and how each reason is worded.

use crate::launch::driver_path::DriverLocation;
use crate::launch::sysroot::format_stderr;
use crate::report::{INSTALL_RUSTUP_HELP, RUN_SETUP_HELP};
use crate::toolchain::ToolchainError;

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

/// The message for a preflight `failure`. `toolchain` is the toolchain the preflight queried,
/// `driver` the driver it ran, and `frontend_version` this front-end's version.
///
/// Every message about the driver names its path and where it was found, and every failure
/// `setup` can repair ends with a `help` naming it. A missing rustup is the exception: `setup`
/// needs rustup too, so that message's `help` names the two ways forward instead.
pub fn failure_message(
    failure: &PreflightFailure,
    toolchain: &str,
    driver: &DriverLocation,
    frontend_version: &str,
) -> String {
    let setup = format!("\n\n{RUN_SETUP_HELP}");
    let driver = driver.describe();

    match failure {
        PreflightFailure::RustupMissing => format!(
            "rustup was not found on PATH, and cargo-cgp installs and selects its toolchain \
             `{toolchain}` through rustup\n\n{INSTALL_RUSTUP_HELP}"
        ),
        PreflightFailure::ToolchainUnavailable { stderr } => format!(
            "toolchain `{toolchain}` is not available{}{setup}",
            format_stderr(stderr.as_bytes())
        ),
        PreflightFailure::DriverDidNotStart { error } => {
            format!("failed to run {driver}: {error}{setup}")
        }
        PreflightFailure::DriverDidNotRun { stderr } => format!(
            "{driver} could not run under toolchain `{toolchain}` (it was likely built against a \
             different nightly){}{setup}",
            format_stderr(stderr.as_bytes())
        ),
        PreflightFailure::UnparsableVersion => {
            format!("could not parse the `--version` output of {driver}{setup}")
        }
        PreflightFailure::VersionMismatch { driver_version } => format!(
            "{driver} is version {driver_version}, but this cargo-cgp is {frontend_version} \
             (the two are out of lockstep){setup}"
        ),
        PreflightFailure::RustcMismatch {
            built_against,
            installed,
        } => format!(
            "{driver} was built against `{built_against}`, but toolchain `{toolchain}` provides \
             `{installed}`{setup}"
        ),
    }
}
