//! Discovering the toolchain sysroot.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, bail};

/// Query the active toolchain's sysroot by running `rustc --print sysroot`.
///
/// The driver links `rustc_driver` dynamically, so at runtime it needs both the sysroot
/// libraries on the dynamic-linker path and an explicit `--sysroot` (rustc cannot infer
/// one from the driver's own out-of-tree location). We resolve the sysroot once here and
/// hand it to the driver rather than making the driver shell out again.
///
/// `rustc` is taken as a parameter so the caller controls which compiler is queried.
pub fn sysroot(rustc: &str) -> anyhow::Result<PathBuf> {
    let output = Command::new(rustc)
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
