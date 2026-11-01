//! Building the binaries under test.

use std::process::Command;

use crate::harness::cargo::cargo;

/// Build the `cargo-cgp` front-end and its driver into the workspace `target/debug`.
/// Both are needed: the front-end locates the driver as its sibling.
pub fn build_binaries() {
    let status = Command::new(cargo())
        // The workspace root holds `rust-toolchain.toml`, so building there selects the pinned
        // toolchain: the one whose compiler the driver embeds.
        .current_dir(crate::paths::workspace_root())
        .args([
            "build",
            "-q",
            "--bin",
            "cargo-cgp",
            "--bin",
            "cargo-cgp-driver",
        ])
        .status()
        .expect("running `cargo build` for the cargo-cgp binaries");
    assert!(status.success(), "failed to build the cargo-cgp binaries");
}
