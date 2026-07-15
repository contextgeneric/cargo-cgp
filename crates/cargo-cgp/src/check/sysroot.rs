//! Discovering the toolchain sysroot.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, bail};

use crate::config::RUSTUP_TOOLCHAIN_ENV;

/// Query a toolchain's sysroot by running `rustc --print sysroot`.
///
/// The driver links `rustc_driver` dynamically, so at runtime it needs both the sysroot
/// libraries on the dynamic-linker path and an explicit `--sysroot` (rustc cannot infer
/// one from the driver's own out-of-tree location). We resolve the sysroot once here and
/// hand it to the driver rather than making the driver shell out again.
///
/// `rustc` is the compiler to query. When `toolchain` is `Some`, `RUSTUP_TOOLCHAIN` is
/// forced to it so the *pinned* sysroot is returned regardless of the project's own
/// toolchain; when `None` (unmanaged mode), the ambient toolchain's sysroot is used.
pub fn sysroot(rustc: &str, toolchain: Option<&str>) -> anyhow::Result<PathBuf> {
    let mut command = Command::new(rustc);
    if let Some(toolchain) = toolchain {
        command.env(RUSTUP_TOOLCHAIN_ENV, toolchain);
    }

    let output = command
        .arg("--print")
        .arg("sysroot")
        .output()
        .with_context(|| format!("failed to run `{rustc} --print sysroot`"))?;

    if !output.status.success() {
        bail!(
            "`{rustc} --print sysroot` failed with status {}",
            output.status
        );
    }

    let path = String::from_utf8(output.stdout)
        .context("`rustc --print sysroot` produced non-UTF-8 output")?;

    Ok(PathBuf::from(path.trim()))
}
