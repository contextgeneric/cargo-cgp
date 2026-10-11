//! The isolated target directory a wrapped build uses.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::config::{CHECK_TARGET_DIR, CHECK_TARGET_SUBDIR, RUSTUP_TOOLCHAIN_ENV};

/// Add `--target-dir <dir>` unless the caller forwarded a `--target-dir` (in either
/// `--target-dir X` or `--target-dir=X` form), which is taken as given.
///
/// The directory is the `cgp` subdirectory of the target directory cargo itself would use, so it
/// lands beside the project's normal build wherever the command is run from, and follows a
/// `CARGO_TARGET_DIR`, a `build.target-dir` in a config file, or one given with `--config`. A
/// separate directory keeps the check from waiting on cargo's lock of the directory every other
/// cargo run in the project holds. When cargo cannot say (no package here, or a broken manifest),
/// the relative [`CHECK_TARGET_DIR`] is used and the wrapped command reports the problem itself.
pub fn inject_target_dir(command: &mut Command, forwarded_args: &[String]) {
    if forwards_target_dir(forwarded_args) {
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
    cargo_args(args)
        .iter()
        .any(|arg| arg == "--target-dir" || arg.starts_with("--target-dir="))
}

/// The arguments cargo itself reads: those before a `--`, after which they belong to rustc.
fn cargo_args(args: &[String]) -> &[String] {
    match args.iter().position(|arg| arg == "--") {
        Some(separator) => &args[..separator],
        None => args,
    }
}

/// The `--manifest-path` the caller forwarded, in either `--manifest-path X` or
/// `--manifest-path=X` form, so the target-directory query asks about the same package the
/// wrapped command builds.
pub fn forwarded_manifest_path(args: &[String]) -> Option<String> {
    let mut args = cargo_args(args).iter();
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
    command.args(metadata_query_args(forwarded_args));
    if let Some((_, Some(toolchain))) = wrapped
        .get_envs()
        .find(|(key, _)| *key == RUSTUP_TOOLCHAIN_ENV)
    {
        command.env(RUSTUP_TOOLCHAIN_ENV, toolchain);
    }
    let output = command.stderr(Stdio::null()).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_target_directory(&String::from_utf8_lossy(&output.stdout))
}

/// The `cargo metadata` arguments that ask for the target directory of the package the forwarded
/// arguments select.
pub fn metadata_query_args(forwarded_args: &[String]) -> Vec<String> {
    let mut args: Vec<String> = ["metadata", "--no-deps", "--format-version", "1"]
        .map(str::to_owned)
        .to_vec();
    if let Some(manifest_path) = forwarded_manifest_path(forwarded_args) {
        args.push("--manifest-path".to_owned());
        args.push(manifest_path);
    }
    for config in forwarded_config(forwarded_args) {
        args.push("--config".to_owned());
        args.push(config);
    }
    args
}

/// The values of every `--config` the caller forwarded, in either `--config X` or `--config=X`
/// form, in order. A `build.target-dir` among them decides the target directory, and any other
/// value may change how cargo reads the project, so the query sees them all.
pub fn forwarded_config(args: &[String]) -> Vec<String> {
    let mut values = Vec::new();
    let mut args = cargo_args(args).iter();
    while let Some(arg) = args.next() {
        if arg == "--config" {
            values.extend(args.next().cloned());
        } else if let Some(value) = arg.strip_prefix("--config=") {
            values.push(value.to_owned());
        }
    }
    values
}
