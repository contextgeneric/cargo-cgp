//! The OS dynamic-library search path.
//!
//! The driver links `librustc_driver` from the sysroot, so that directory must be on the
//! loader's search path whenever the driver runs — both when cargo spawns it for the check
//! and when the preflight runs `cargo-cgp-driver --version` as a load test.

use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

/// Name of the OS variable that lists directories searched for dynamic libraries.
#[cfg(target_os = "macos")]
pub const DYLIB_PATH_VAR: &str = "DYLD_FALLBACK_LIBRARY_PATH";
#[cfg(target_os = "windows")]
pub const DYLIB_PATH_VAR: &str = "PATH";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const DYLIB_PATH_VAR: &str = "LD_LIBRARY_PATH";

/// Prepend `dir` to the dynamic-library search path of `command`, preserving any existing
/// value, so the spawned driver can load `librustc_driver` from the sysroot.
pub fn prepend_dylib_path(command: &mut Command, dir: &Path) {
    let mut entries = vec![OsString::from(dir)];
    if let Some(existing) = env::var_os(DYLIB_PATH_VAR) {
        entries.extend(env::split_paths(&existing).map(OsString::from));
    }

    let joined = env::join_paths(entries).expect("sysroot lib path contains an invalid character");
    command.env(DYLIB_PATH_VAR, joined);
}
