//! The OS dynamic-library search path.
//!
//! The driver links `librustc_driver` from the sysroot, so that directory must be on the
//! loader's search path whenever the driver runs — both when cargo spawns it for the check
//! and when the preflight runs `cargo-cgp-driver --version` as a load test.

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;

use anyhow::Context;

/// Name of the OS variable that lists directories searched for dynamic libraries.
#[cfg(target_os = "macos")]
pub const DYLIB_PATH_VAR: &str = "DYLD_FALLBACK_LIBRARY_PATH";
#[cfg(target_os = "windows")]
pub const DYLIB_PATH_VAR: &str = "PATH";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const DYLIB_PATH_VAR: &str = "LD_LIBRARY_PATH";

/// Prepend `dir` to the dynamic-library search path of `command`, preserving any existing
/// value, so the spawned driver can load `librustc_driver` from the sysroot.
pub fn prepend_dylib_path(command: &mut Command, dir: &Path) -> anyhow::Result<()> {
    let existing = env::var_os(DYLIB_PATH_VAR);
    command.env(DYLIB_PATH_VAR, dylib_search_path(dir, existing.as_deref())?);
    Ok(())
}

/// The search path with `dir` in front of the `existing` entries. An error when `dir` cannot be
/// one entry of the list, which happens when it contains the list separator (`:` on Unix).
pub fn dylib_search_path(dir: &Path, existing: Option<&OsStr>) -> anyhow::Result<OsString> {
    let mut entries = vec![OsString::from(dir)];
    if let Some(existing) = existing {
        entries.extend(env::split_paths(existing).map(OsString::from));
    }

    env::join_paths(entries).with_context(|| {
        format!(
            "the toolchain library directory {} cannot be added to {DYLIB_PATH_VAR}",
            dir.display()
        )
    })
}
