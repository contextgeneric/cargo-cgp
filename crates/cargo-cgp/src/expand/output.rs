//! Where the driver writes the expansion, and how the front-end reads it back.

use std::path::PathBuf;
use std::{env, fs, process};

/// The file the driver writes the finished expansion to, and the front-end then prints.
///
/// The content travels through a file rather than the driver's stdout so it cannot interleave
/// with cargo's own progress output — the same reason `cargo-expand` passes `-o`. The name
/// carries the front-end's process id, so two concurrent runs never read each other's output.
pub fn output_path() -> PathBuf {
    env::temp_dir().join(format!("cargo-cgp-expand-{}.rs", process::id()))
}

/// Read the expansion the driver wrote, if it wrote one.
///
/// `None` means the driver never reached expand mode — a build error before expansion, or a
/// cargo invocation that compiled no workspace target — which the caller reports rather than
/// printing nothing. A stale file cannot be mistaken for output: [`output_path`] is unique per
/// process, and the caller clears it before launching cargo.
pub fn read_expansion(path: &PathBuf) -> Option<String> {
    let expansion = fs::read_to_string(path).ok()?;
    (!expansion.trim().is_empty()).then_some(expansion)
}
