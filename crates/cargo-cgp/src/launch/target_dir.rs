//! The isolated target directory a wrapped build uses.

use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::config::{CHECK_TARGET_DIR, CHECK_TARGET_SUBDIR, RUSTUP_TOOLCHAIN_ENV};

/// Add `--target-dir <dir>` unless the caller already chose the target directory — via a
/// forwarded `--target-dir` (in either `--target-dir X` or `--target-dir=X` form) or the
/// `CARGO_TARGET_DIR` environment variable, both of which take precedence.
///
/// The directory is the `cgp` subdirectory of the target directory cargo itself would use, so it
/// lands beside the project's normal build wherever the command is run from, and follows a
/// `build.target-dir` configured for the project. When cargo cannot say (no package here, or a
/// broken manifest), the relative [`CHECK_TARGET_DIR`] is used and the wrapped command reports
/// the problem itself.
pub fn inject_target_dir(command: &mut Command, forwarded_args: &[String]) {
    if env::var_os("CARGO_TARGET_DIR").is_some() || forwards_target_dir(forwarded_args) {
        return;
    }
    let dir = cargo_target_directory(command, forwarded_args)
        .map(|dir| dir.join(CHECK_TARGET_SUBDIR))
        .unwrap_or_else(|| PathBuf::from(CHECK_TARGET_DIR));
    command.arg("--target-dir").arg(dir);
}

/// Whether the forwarded arguments already set `--target-dir` (in either `--target-dir X`
/// or `--target-dir=X` form), in which case the default is not injected.
pub fn forwards_target_dir(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--target-dir" || arg.starts_with("--target-dir="))
}

/// The `--manifest-path` the caller forwarded, in either `--manifest-path X` or
/// `--manifest-path=X` form, so the target-directory query asks about the same package the
/// wrapped command builds.
pub fn forwarded_manifest_path(args: &[String]) -> Option<String> {
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--manifest-path" {
            return args.next().cloned();
        }
        if let Some(path) = arg.strip_prefix("--manifest-path=") {
            return Some(path.to_owned());
        }
    }
    None
}

/// The `target_directory` field of `cargo metadata`'s JSON output, or `None` when the output is
/// not that JSON.
pub fn parse_target_directory(metadata_json: &str) -> Option<PathBuf> {
    let value: serde_json::Value = serde_json::from_str(metadata_json).ok()?;
    value.get("target_directory")?.as_str().map(PathBuf::from)
}

/// Ask cargo for the target directory of the package the forwarded arguments select. Runs
/// `cargo metadata --no-deps`, which reads the manifests and configuration without resolving or
/// fetching dependencies or invoking `rustc`; any failure is `None`, and its message is discarded
/// because the wrapped command meets the same problem and reports it.
///
/// The query runs under the toolchain the wrapped command was given, by copying its
/// `RUSTUP_TOOLCHAIN`, so rustup's `cargo` proxy never resolves the project's own toolchain file,
/// which could otherwise make rustup install a toolchain the check itself never uses.
fn cargo_target_directory(wrapped: &Command, forwarded_args: &[String]) -> Option<PathBuf> {
    let mut command = Command::new("cargo");
    command.args(["metadata", "--no-deps", "--format-version", "1"]);
    if let Some((_, Some(toolchain))) = wrapped
        .get_envs()
        .find(|(key, _)| *key == RUSTUP_TOOLCHAIN_ENV)
    {
        command.env(RUSTUP_TOOLCHAIN_ENV, toolchain);
    }
    if let Some(manifest_path) = forwarded_manifest_path(forwarded_args) {
        command.arg("--manifest-path").arg(manifest_path);
    }
    let output = command.stderr(Stdio::null()).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_target_directory(&String::from_utf8_lossy(&output.stdout))
}
