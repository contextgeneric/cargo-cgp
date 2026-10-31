//! Reinstalling the front-end and handing off to its new `setup`.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, bail};

use crate::config::FRONTEND_CRATE;
use crate::exit::exit_code;

/// Reinstall the front-end from crates.io at exactly `version`. The version is named because a
/// bare `cargo install` picks the newest *stable* release, which is the wrong target for a
/// pre-release install moving to a newer pre-release. On Windows the running binary is locked
/// and this fails; we translate that into the manual commands the user can run from a shell
/// where `cargo-cgp` is not running.
pub fn reinstall_frontend(version: &str) -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .args(frontend_install_args(version))
        .status()
        .context("failed to run `cargo install` (is cargo on PATH?)")?;

    if !status.success() {
        bail!(
            "`cargo install {FRONTEND_CRATE} --version {version} --locked` failed ({status})\n\n\
             note: on Windows the running executable is locked and cannot be replaced in place\n\
             help: from a shell where cargo-cgp is not running, update by hand with:\n\
             \x20   cargo install {FRONTEND_CRATE} --version {version} --locked\n\
             \x20   cargo cgp setup"
        );
    }
    Ok(())
}

/// The `cargo install` arguments that install the front-end at `version`. `--locked` builds the
/// published lockfile, as `setup` does for the driver.
pub fn frontend_install_args(version: &str) -> Vec<String> {
    ["install", FRONTEND_CRATE, "--version", version, "--locked"]
        .map(str::to_owned)
        .to_vec()
}

/// Run the freshly installed front-end's `setup`. `exe` is the path the front-end was installed
/// at, read before the reinstall; the binary there is now the new version (cargo replaced the
/// file), so its `setup` knows the new pinned toolchain.
pub fn run_new_setup(exe: &Path) -> anyhow::Result<i32> {
    let status = Command::new(exe)
        .arg("setup")
        .status()
        .context("failed to run the updated `cargo cgp setup`")?;
    Ok(exit_code(status))
}
