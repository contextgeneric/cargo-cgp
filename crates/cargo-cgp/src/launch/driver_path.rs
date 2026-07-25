//! Locating the driver executable.

use std::env;
use std::path::PathBuf;

use anyhow::Context;

use crate::config::DRIVER_ENV;

/// Resolve the path to the `cargo-cgp-driver` executable.
///
/// An explicit [`DRIVER_ENV`] override wins, so local development and the UI test harness
/// can point at a freshly built driver. Otherwise, since cargo and rustup lay the driver
/// next to the `cargo-cgp` binary that is running (both end up in the same
/// `target/<profile>` or `~/.cargo/bin` directory), we look for a sibling of the current
/// executable. If that sibling is missing we fall back to the bare name and let the OS
/// resolve it on `PATH`.
///
/// `driver_bin` is the executable name ([`crate::config::DRIVER_BIN`]); the platform
/// executable suffix (e.g. `.exe`) is appended here.
pub fn driver_path(driver_bin: &str) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = env::var_os(DRIVER_ENV) {
        return Ok(PathBuf::from(explicit));
    }

    let file_name = format!("{driver_bin}{}", env::consts::EXE_SUFFIX);

    let current =
        env::current_exe().context("failed to locate the running cargo-cgp executable")?;

    if let Some(dir) = current.parent() {
        let sibling = dir.join(&file_name);
        if sibling.is_file() {
            return Ok(sibling);
        }
    }

    // Not found beside us — defer resolution to PATH.
    Ok(PathBuf::from(file_name))
}
