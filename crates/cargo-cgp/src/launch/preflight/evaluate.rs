//! Deciding on what a driver reported.

use crate::launch::preflight::{DriverVersion, PreflightFailure, parse_driver_version};

/// Judge the answer of a driver that started for `--version`: whether it ran (`succeeded`), and
/// what it printed. A failed run keeps its stderr, the loader's reason; a run whose output lacks
/// the driver's version line or its `built-against-rustc:` line is unparsable; otherwise
/// [`evaluate`] compares its identity.
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
