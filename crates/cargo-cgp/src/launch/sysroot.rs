//! Discovering the toolchain sysroot.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, bail};

use crate::report::quoted_output;

/// Query a toolchain's sysroot by running `rustc --print sysroot`.
///
/// The driver links `rustc_driver` dynamically, so at runtime it needs both the sysroot
/// libraries on the dynamic-linker path and an explicit `--sysroot` (rustc cannot infer
/// one from the driver's own out-of-tree location). We resolve the sysroot once here and
/// hand it to the driver rather than making the driver shell out again.
///
/// `rustc` is the compiler to query. When `toolchain` is `Some`, it runs as
/// `rustup run <toolchain> <rustc>`, so the *pinned* sysroot is returned regardless of the
/// project's own toolchain or another `rustc` earlier on `PATH`; when `None` (unmanaged mode),
/// the ambient compiler's sysroot is used.
pub fn sysroot(rustc: &str, toolchain: Option<&str>) -> anyhow::Result<PathBuf> {
    let (program, args) = sysroot_command(rustc, toolchain);
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("failed to run `{rustc} --print sysroot`"))?;

    if !output.status.success() {
        // Include the compiler's own stderr: when this probe fails it is almost never
        // rustc rejecting the query but the process failing to start, and the reason is
        // only ever in that stream. A dynamic-loader failure is the archetype: it exits
        // 127 with `error while loading shared libraries: …` on stderr, which the status
        // alone reduces to an unexplained number.
        bail!(
            "`{rustc} --print sysroot` failed ({}){}",
            output.status,
            format_stderr(&output.stderr)
        );
    }

    let path = String::from_utf8(output.stdout)
        .context("`rustc --print sysroot` produced non-UTF-8 output")?;

    Ok(PathBuf::from(path.trim()))
}

/// The program and arguments [`sysroot`] runs: `<rustc> --print sysroot`, through
/// `rustup run <toolchain>` when a toolchain is forced.
pub fn sysroot_command(rustc: &str, toolchain: Option<&str>) -> (String, Vec<String>) {
    let print = ["--print", "sysroot"].map(str::to_owned);
    match toolchain {
        Some(toolchain) => (
            "rustup".to_owned(),
            ["run", toolchain, rustc]
                .map(str::to_owned)
                .into_iter()
                .chain(print)
                .collect(),
        ),
        None => (rustc.to_owned(), print.to_vec()),
    }
}

/// A failed probe's stderr as a quoted block to append to the error ([`quoted_output`]), or
/// nothing when it is empty. Lossy-decoded, since a loader message is written by the OS rather
/// than by rustc and is not guaranteed to be UTF-8, and a mangled byte is no reason to withhold
/// the diagnosis.
pub fn format_stderr(stderr: &[u8]) -> String {
    quoted_output(&String::from_utf8_lossy(stderr))
}
