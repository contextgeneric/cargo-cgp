//! `cargo cgp setup` — provisioning the toolchain and the driver.
//!
//! This is the one command that does the heavyweight, stateful work `check` refuses to do:
//! install the pinned nightly (with the `rustc-dev` and `llvm-tools` components the driver
//! links) and build the driver against it, landing the driver beside the front-end so the
//! sibling lookup finds it. It reads the pinned toolchain and the target version from the
//! front-end's own constants, so the user never types a nightly date or a version.

use std::env;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, bail};

use crate::config::{DRIVER_CRATE, RUSTUP_TOOLCHAIN_ENV, TOOL_VERSION};
use crate::toolchain::pinned_toolchain;

/// Install the pinned toolchain and the matching driver. Returns the exit code to
/// propagate (0 on success). Progress streams to the terminal, since both steps can take a
/// while.
pub fn run_setup(_args: &[String]) -> anyhow::Result<i32> {
    let toolchain = pinned_toolchain();

    eprintln!("cargo-cgp: installing toolchain `{toolchain}` (with rustc-dev, llvm-tools)…");
    install_toolchain(&toolchain)?;

    eprintln!("cargo-cgp: installing {DRIVER_CRATE} {TOOL_VERSION} under `{toolchain}`…");
    install_driver(&toolchain)?;

    eprintln!("cargo-cgp: setup complete.");
    Ok(0)
}

/// Install the toolchain with the components the driver needs, via rustup. Idempotent:
/// rustup makes it a no-op when the toolchain and components are already present.
fn install_toolchain(toolchain: &str) -> anyhow::Result<()> {
    let status = Command::new("rustup")
        .args(["toolchain", "install", toolchain])
        .args(["-c", "rustc-dev", "-c", "llvm-tools"])
        .status()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                anyhow::anyhow!(
                    "rustup was not found on PATH; cargo-cgp requires rustup to manage toolchains"
                )
            } else {
                anyhow::Error::new(e).context("failed to run `rustup toolchain install`")
            }
        })?;

    if !status.success() {
        bail!("`rustup toolchain install {toolchain}` failed with status {status}");
    }
    Ok(())
}

/// Build and install the driver under `toolchain`, at the front-end's own version, next to
/// the front-end. `cargo install` is idempotent here: it exits successfully (a no-op) when
/// that exact version is already installed, and replaces an older one when it is not.
fn install_driver(toolchain: &str) -> anyhow::Result<()> {
    let mut command = Command::new("cargo");
    command
        .env(RUSTUP_TOOLCHAIN_ENV, toolchain)
        .arg("install")
        .arg(DRIVER_CRATE)
        .args(["--version", TOOL_VERSION]);

    if let Some(root) = install_root() {
        command.arg("--root").arg(root);
    }

    let status = command
        .status()
        .context("failed to run `cargo install` for the driver (is cargo on PATH?)")?;

    if !status.success() {
        bail!("installing {DRIVER_CRATE} {TOOL_VERSION} failed with status {status}");
    }
    Ok(())
}

/// The `--root` for `cargo install` so the driver lands *next to* the front-end, since the
/// sibling lookup expects them together. `cargo install` places binaries in `<root>/bin`,
/// so when the front-end sits in a `bin` directory (the usual `~/.cargo/bin/cargo-cgp`) the
/// root is that directory's parent. When it sits elsewhere, we return `None` and let cargo
/// use its default root, warning the user, rather than guessing a wrong location.
fn install_root() -> Option<PathBuf> {
    let current = env::current_exe().ok()?;
    let dir = current.parent()?;
    if dir.file_name().is_some_and(|name| name == "bin") {
        return dir.parent().map(PathBuf::from);
    }
    eprintln!(
        "cargo-cgp: warning: cargo-cgp is not in a `bin` directory ({}); \
         installing the driver to cargo's default location instead of beside it",
        dir.display()
    );
    None
}
