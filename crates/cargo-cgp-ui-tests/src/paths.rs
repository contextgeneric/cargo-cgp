//! Locating the workspace, the fixtures, the cgp checkout, and the built binaries.
//!
//! The harness is a separate crate, so it cannot use `CARGO_BIN_EXE_*` (those name only
//! the binaries of the crate under test). Instead it finds the `target/debug` directory
//! from its own executable location, which is robust to `CARGO_TARGET_DIR`, and resolves
//! everything else relative to the workspace root.

use std::env;
use std::path::{Path, PathBuf};

/// The cargo-cgp workspace root — two levels up from this crate's manifest
/// (`crates/cargo-cgp-ui-tests`).
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolving the workspace root")
}

/// The UI fixture tree, `tests/ui/` under the workspace root.
pub fn fixtures_dir() -> PathBuf {
    workspace_root().join("tests/ui")
}

/// The cgp facade crate, assumed to live in a sibling `cgp` checkout at `../cgp`.
pub fn cgp_crate_dir() -> PathBuf {
    workspace_root()
        .join("../cgp/crates/main/cgp")
        .canonicalize()
        .expect("resolving the cgp crate (expected a sibling ../cgp checkout)")
}

/// The root of the cgp checkout (`crates/main/cgp` stripped from [`cgp_crate_dir`]). Its
/// absolute path appears in cross-crate diagnostic notes, so snapshots normalize it away.
pub fn cgp_root() -> PathBuf {
    cgp_crate_dir()
        .ancestors()
        .nth(3)
        .expect("deriving the cgp checkout root")
        .to_path_buf()
}

/// The `target/debug` directory, derived from the harness test binary's own path
/// (`<target>/debug/deps/ui-<hash>`).
pub fn debug_dir() -> PathBuf {
    let exe = env::current_exe().expect("resolving the harness executable path");
    exe.parent()
        .and_then(Path::parent)
        .expect("deriving target/debug from the harness executable path")
        .to_path_buf()
}

/// Path to the built `cargo-cgp` front-end binary.
pub fn cargo_cgp_bin() -> PathBuf {
    debug_dir().join(format!("cargo-cgp{}", env::consts::EXE_SUFFIX))
}
