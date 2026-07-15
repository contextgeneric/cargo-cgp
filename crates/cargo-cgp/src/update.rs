//! `cargo cgp update` — upgrading the tool to its latest published version.
//!
//! Update is a thin orchestrator over cargo. It reads the crates.io sparse index for the
//! front-end crate, picks the highest published version **in the running version's channel**
//! (stable stays on stable, a pre-release stays on pre-releases), and stops early unless
//! that is strictly newer. Otherwise it reinstalls the front-end with `cargo install` and
//! hands off to the freshly installed `cargo cgp setup`, which brings the driver and
//! toolchain up to the new version.
//!
//! There is no self-replacing installer: on Unix `cargo install` atomically replaces the
//! running binary, and on Windows — where the running executable is locked — the reinstall
//! fails and update prints the commands to run by hand instead.
//!
//! Enumerating *all* versions (rather than taking the single "latest" cargo would report)
//! is what makes channel preservation possible: `cargo search`/`cargo info` only report the
//! max version, which includes pre-releases, so a stable install could not otherwise be
//! offered the latest stable when a higher pre-release also exists. The sparse-index access
//! here follows the shape of the `crates-index` crate but is written directly over the
//! widely-used `ureq` and `serde_json` rather than taking on that dependency.

use std::env;
use std::process::Command;

use anyhow::{Context, bail};
use semver::Version;

use crate::config::{FRONTEND_CRATE, TOOL_VERSION};

/// Find a newer in-channel version and, if there is one, reinstall the front-end and run
/// the new `setup`. Returns the exit code to propagate.
pub fn run_update(_args: &[String]) -> anyhow::Result<i32> {
    let index = fetch_sparse_index(FRONTEND_CRATE)?;
    let versions = parse_versions(&index);

    let Some(latest) = select_update(&versions, TOOL_VERSION)? else {
        println!("cargo-cgp is already up to date (v{TOOL_VERSION})");
        return Ok(0);
    };

    println!("cargo-cgp: updating v{TOOL_VERSION} → v{latest}…");
    reinstall_frontend()?;

    // Hand off to the newly installed front-end's setup, which knows the new pinned
    // toolchain and provisions the matching driver.
    run_new_setup()
}

/// Read the crates.io sparse index entry for `crate_name` over HTTP and return the body: one
/// JSON object per line, one per published version.
fn fetch_sparse_index(crate_name: &str) -> anyhow::Result<String> {
    let url = format!("https://index.crates.io/{}", sparse_index_path(crate_name));
    let user_agent = format!("cargo-cgp/{TOOL_VERSION}");

    ureq::get(url.as_str())
        .header("User-Agent", &user_agent)
        .call()
        .with_context(|| format!("failed to query the crates.io index at {url}"))?
        .body_mut()
        .read_to_string()
        .context("failed to read the crates.io index response")
}

/// The sparse-index path for a crate, following the registry's directory convention: one
/// segment for names of 1–2 characters, `3/<first>` for 3, and `<first two>/<next two>` for
/// 4 or more. Crate names are ASCII, so byte slicing is safe.
pub fn sparse_index_path(crate_name: &str) -> String {
    let name = crate_name.to_ascii_lowercase();
    let dir = match name.len() {
        0 => String::new(),
        1 => "1".to_owned(),
        2 => "2".to_owned(),
        3 => format!("3/{}", &name[0..1]),
        _ => format!("{}/{}", &name[0..2], &name[2..4]),
    };
    format!("{dir}/{name}")
}

/// Extract the non-yanked version strings from a sparse-index body. Each line is a JSON
/// object with `vers` and `yanked`; a line that does not parse or lacks `vers` is skipped,
/// and a yanked version is dropped so an update never lands on one.
pub fn parse_versions(index_body: &str) -> Vec<String> {
    index_body
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let value: serde_json::Value = serde_json::from_str(line).ok()?;
            if value.get("yanked").and_then(serde_json::Value::as_bool) == Some(true) {
                return None;
            }
            value
                .get("vers")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .collect()
}

/// Pick the highest version that is both **in `current`'s channel** and strictly newer than
/// it, or `None` when there is nothing to move to. The channel is preserved by matching
/// pre-release-ness: a stable current (no pre-release) considers only stable candidates, so
/// it never jumps to a pre-release, and a pre-release current considers only pre-releases.
pub fn select_update(versions: &[String], current: &str) -> anyhow::Result<Option<String>> {
    let current = Version::parse(current)
        .with_context(|| format!("`{current}` is not a valid semver version"))?;
    let want_prerelease = !current.pre.is_empty();

    let best = versions
        .iter()
        .filter_map(|version| Version::parse(version).ok())
        .filter(|version| !version.pre.is_empty() == want_prerelease)
        .filter(|version| *version > current)
        .max();

    Ok(best.map(|version| version.to_string()))
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
