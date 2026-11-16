//! Resolving and querying the pinned toolchain: [`PINNED_TOOLCHAIN`] or the [`TOOLCHAIN_ENV`]
//! override, queried through rustup; see
//! `cgp-knowledge-base/cargo-cgp/implementation/distribution/provisioning.md`.

use std::io::ErrorKind;
use std::process::Command;

use crate::config::{PINNED_TOOLCHAIN, TOOLCHAIN_ENV};

/// The toolchain cargo-cgp manages: the [`TOOLCHAIN_ENV`] override if set, else the
/// [`PINNED_TOOLCHAIN`] baked in at build time.
pub fn pinned_toolchain() -> String {
    std::env::var(TOOLCHAIN_ENV).unwrap_or_else(|_| PINNED_TOOLCHAIN.to_owned())
}

/// Why [`rustc_version`] could not read a toolchain's compiler.
#[derive(Debug)]
pub enum ToolchainError {
    /// `rustup` could not be started, so no toolchain can be queried or installed.
    RustupMissing,
    /// rustup ran but the query failed, usually because the toolchain is not installed.
    QueryFailed {
        /// rustup's stderr, which says why.
        stderr: String,
    },
    /// Any other failure to run the query or read its output.
    Other(anyhow::Error),
}

/// The `rustc --version` line of `toolchain`, run as `rustup run <toolchain> rustc --version`; see
/// `cgp-knowledge-base/cargo-cgp/implementation/distribution/provisioning.md`.
pub fn rustc_version(toolchain: &str) -> Result<String, ToolchainError> {
    let output = Command::new("rustup")
        .args(["run", toolchain, "rustc", "--version"])
        .output()
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => ToolchainError::RustupMissing,
            _ => ToolchainError::Other(
                anyhow::Error::new(error).context("failed to run `rustup run … rustc --version`"),
            ),
        })?;

    if !output.status.success() {
        return Err(ToolchainError::QueryFailed {
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    String::from_utf8(output.stdout)
        .map(|version| version.trim().to_owned())
        .map_err(|_| {
            ToolchainError::Other(anyhow::anyhow!(
                "`rustc --version` produced non-UTF-8 output"
            ))
        })
}
