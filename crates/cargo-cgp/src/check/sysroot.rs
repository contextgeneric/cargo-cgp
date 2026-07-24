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
        // Include the compiler's own stderr: when this probe fails it is almost never
        // rustc rejecting the query but the process failing to start, and the reason is
        // only ever in that stream. A dynamic-loader failure is the archetype — it exits
        // 127 with `error while loading shared libraries: …` on stderr — which the status
        // alone reduces to an unexplained number.
        bail!(
            "`{rustc} --print sysroot` failed with status {}{}",
            output.status,
            format_stderr(&output.stderr)
        );
    }

    let path = String::from_utf8(output.stdout)
        .context("`rustc --print sysroot` produced non-UTF-8 output")?;

    Ok(PathBuf::from(path.trim()))
}

/// A failed probe's stderr as a block to append to the error, or nothing when it is empty.
/// Lossy-decoded, since a loader message is written by the OS rather than by rustc and is
/// not guaranteed to be UTF-8 — and a mangled byte is no reason to withhold the diagnosis.
pub fn format_stderr(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let text = text.trim();
    if text.is_empty() {
        String::new()
    } else {
        format!(":\n\n{text}")
    }
}
