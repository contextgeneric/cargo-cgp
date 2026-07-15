//! Resolving and querying the pinned toolchain.
//!
//! The driver embeds one exact nightly, so `cargo cgp check` forces that toolchain for the
//! wrapped compilation and `setup` installs it. This module names the effective toolchain
//! (the [`PINNED_TOOLCHAIN`] constant, or the [`TOOLCHAIN_ENV`] override) and queries its
//! `rustc` through rustup, which is also how the preflight learns whether the toolchain is
//! installed at all.

use std::process::Command;

use anyhow::{Context, bail};

use crate::config::{PINNED_TOOLCHAIN, RUSTUP_TOOLCHAIN_ENV, TOOLCHAIN_ENV};

/// The toolchain cargo-cgp manages: the [`TOOLCHAIN_ENV`] override if set, else the
/// [`PINNED_TOOLCHAIN`] baked in at build time.
pub fn pinned_toolchain() -> String {
    std::env::var(TOOLCHAIN_ENV).unwrap_or_else(|_| PINNED_TOOLCHAIN.to_owned())
}

/// The `rustc --version` line of `toolchain`, run through rustup by forcing
/// `RUSTUP_TOOLCHAIN`. A failure means the toolchain is not installed (or rustup is
/// absent), which the preflight reports as a "run `cargo cgp setup`" case.
pub fn rustc_version(toolchain: &str) -> anyhow::Result<String> {
    let output = Command::new("rustc")
        .env(RUSTUP_TOOLCHAIN_ENV, toolchain)
        .arg("--version")
        .output()
        .context("failed to run `rustc` (is rustup on PATH?)")?;

    if !output.status.success() {
        bail!(
            "toolchain `{toolchain}` is not available ({})",
            output.status
        );
    }

    Ok(String::from_utf8(output.stdout)
        .context("`rustc --version` produced non-UTF-8 output")?
        .trim()
        .to_owned())
}
