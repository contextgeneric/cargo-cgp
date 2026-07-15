//! `cargo cgp update` — upgrading the tool to its latest published version.
//!
//! Update is a thin orchestrator over cargo. It asks crates.io for the latest version and
//! stops early if it is not newer than the running one; otherwise it reinstalls the
//! front-end with `cargo install` and hands off to the freshly installed `cargo cgp setup`,
//! which brings the driver and toolchain up to the new version. There is no self-replacing
//! installer: on Unix `cargo install` atomically replaces the running binary, and on
//! Windows — where the running executable is locked — the reinstall fails and update prints
//! the commands to run by hand instead.

use std::env;
use std::process::Command;

use anyhow::{Context, bail};
use semver::Version;

use crate::config::{FRONTEND_CRATE, TOOL_VERSION};

/// Check for a newer version and, if there is one, reinstall the front-end and run the new
/// `setup`. Returns the exit code to propagate.
pub fn run_update(_args: &[String]) -> anyhow::Result<i32> {
    let latest = query_latest(FRONTEND_CRATE)?;

    if !is_newer(&latest, TOOL_VERSION)? {
        println!("cargo-cgp is already up to date (v{TOOL_VERSION}; latest is v{latest})");
        return Ok(0);
    }

    println!("cargo-cgp: updating v{TOOL_VERSION} → v{latest}…");
    reinstall_frontend()?;

    // Hand off to the newly installed front-end's setup, which knows the new pinned
    // toolchain and provisions the matching driver.
    run_new_setup()
}

/// Ask cargo for the latest published version of `crate_name`. Uses `cargo search` so the
/// query honors the user's configured registry (a mirror or private registry) and needs no
/// HTTP/TLS dependency in this lean binary.
fn query_latest(crate_name: &str) -> anyhow::Result<String> {
    let output = Command::new("cargo")
        .args(["search", crate_name, "--limit", "20"])
        .output()
        .context("failed to run `cargo search` (is cargo on PATH?)")?;

    if !output.status.success() {
        bail!(
            "`cargo search {crate_name}` failed with status {}",
            output.status
        );
    }

    let text =
        String::from_utf8(output.stdout).context("`cargo search` produced non-UTF-8 output")?;

    parse_latest_version(&text, crate_name)
        .with_context(|| format!("could not find `{crate_name}` in `cargo search` output"))
}

/// Extract the version of `crate_name` from `cargo search` output, whose lines read
/// `name = "x.y.z"    # description`. Matches the exact crate name so a substring match on
/// another crate is not mistaken for it.
pub fn parse_latest_version(search_output: &str, crate_name: &str) -> Option<String> {
    let prefix = format!("{crate_name} = \"");
    for line in search_output.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let version = rest.split('"').next()?;
            if !version.is_empty() {
                return Some(version.to_owned());
            }
        }
    }
    None
}

/// Whether `latest` is a strictly newer semver than `current`. Parsing both as semver makes
/// the comparison correct across prerelease labels, so an update never downgrades.
pub fn is_newer(latest: &str, current: &str) -> anyhow::Result<bool> {
    let latest = Version::parse(latest)
        .with_context(|| format!("`{latest}` is not a valid semver version"))?;
    let current = Version::parse(current)
        .with_context(|| format!("`{current}` is not a valid semver version"))?;
    Ok(latest > current)
}

/// Reinstall the front-end from crates.io. On Windows the running binary is locked and this
/// fails; we translate that into the manual commands the user can run from a shell where
/// `cargo-cgp` is not running.
fn reinstall_frontend() -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .args(["install", FRONTEND_CRATE])
        .status()
        .context("failed to run `cargo install` (is cargo on PATH?)")?;

    if !status.success() {
        bail!(
            "`cargo install {FRONTEND_CRATE}` failed with status {status}.\n\
             On Windows the running executable is locked and cannot be replaced in place; \
             from a shell where cargo-cgp is not running, update by hand with:\n\
             \x20   cargo install {FRONTEND_CRATE}\n\
             \x20   cargo cgp setup"
        );
    }
    Ok(())
}

/// Run the freshly installed front-end's `setup`. The binary at this path is now the new
/// version (cargo replaced the file), so its `setup` knows the new pinned toolchain.
fn run_new_setup() -> anyhow::Result<i32> {
    let exe = env::current_exe().context("failed to locate the cargo-cgp executable")?;
    let status = Command::new(exe)
        .arg("setup")
        .status()
        .context("failed to run the updated `cargo cgp setup`")?;
    Ok(status.code().unwrap_or(1))
}
