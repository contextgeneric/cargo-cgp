//! The isolated target directory a wrapped build uses.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::config::{CHECK_TARGET_DIR, CHECK_TARGET_SUBDIR, RUSTUP_TOOLCHAIN_ENV};
use crate::launch::forwarded_flag::{
    forwarded_config, forwarded_manifest_path, forwards_target_dir,
};

/// The target directory the tool chooses for a wrapped command, or `None` when the caller forwarded
/// a `--target-dir`; `wrapped` is the command whose toolchain the query runs under. See
/// `cgp-knowledge-base/cargo-cgp/implementation/executable-structure.md`.
pub fn default_target_dir(wrapped: &Command, forwarded_args: &[String]) -> Option<PathBuf> {
    if forwards_target_dir(forwarded_args) {
        return None;
    }
    Some(cgp_target_dir(cargo_target_directory(
        wrapped,
        forwarded_args,
    )))
}

/// The directory a wrapped build uses given the project's target directory as cargo reports it:
/// its `cgp` subdirectory, or the relative [`CHECK_TARGET_DIR`] when cargo could not say.
pub fn cgp_target_dir(project_target_dir: Option<PathBuf>) -> PathBuf {
    project_target_dir
        .map(|dir| dir.join(CHECK_TARGET_SUBDIR))
        .unwrap_or_else(|| PathBuf::from(CHECK_TARGET_DIR))
}

/// The `target_directory` field of `cargo metadata`'s JSON output, or `None` when the output is
/// not that JSON.
pub fn parse_target_directory(metadata_json: &str) -> Option<PathBuf> {
    let value: serde_json::Value = serde_json::from_str(metadata_json).ok()?;
    value.get("target_directory")?.as_str().map(PathBuf::from)
}

/// Ask cargo, through `cargo metadata --no-deps` under the wrapped command's toolchain, for the
/// target directory of the package the forwarded arguments select; any failure is `None`. See
/// `cgp-knowledge-base/cargo-cgp/implementation/executable-structure.md`.
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
