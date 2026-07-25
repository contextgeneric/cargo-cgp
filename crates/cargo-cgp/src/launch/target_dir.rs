//! The isolated target directory a wrapped build uses.

use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::config::CHECK_TARGET_DIR;

/// Add `--target-dir target/cgp` unless the caller already chose the target directory —
/// via a forwarded `--target-dir` (in either `--target-dir X` or `--target-dir=X` form) or
/// the `CARGO_TARGET_DIR` environment variable, both of which take precedence.
pub fn inject_target_dir(command: &mut Command, forwarded_args: &[String]) {
    if env::var_os("CARGO_TARGET_DIR").is_some() || forwards_target_dir(forwarded_args) {
        return;
    }
    command
        .arg("--target-dir")
        .arg(PathBuf::from(CHECK_TARGET_DIR));
}

/// Whether the forwarded arguments already set `--target-dir` (in either `--target-dir X`
/// or `--target-dir=X` form), in which case the default is not injected.
pub fn forwards_target_dir(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--target-dir" || arg.starts_with("--target-dir="))
}
