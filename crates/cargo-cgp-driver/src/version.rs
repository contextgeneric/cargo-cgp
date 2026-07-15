//! The driver's own `--version` query.
//!
//! When `cargo-cgp-driver` is run directly (not as cargo's rustc wrapper), a `--version`
//! or `-V` flag prints the driver's identity and exits, instead of running the compiler.
//! The front-end's preflight reads this to check the driver is the matching build: the
//! line carries the driver's crate version, the toolchain it is pinned to, and the exact
//! `rustc` it was compiled against (all baked in by `build.rs`).
//!
//! In *wrapper* mode the same flags belong to the real compiler — cargo probes it with
//! `rustc -vV` / `--version` — so [`crate::run`] answers here only when not in wrapper mode.

/// The driver's crate version, kept equal to the front-end's by the shared
/// `[workspace.package]` version.
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The nightly channel the driver is pinned to, from the workspace `rust-toolchain.toml`.
pub const PINNED_TOOLCHAIN: &str = env!("CARGO_CGP_PINNED_TOOLCHAIN");

/// The `rustc --version` line of the compiler that actually built the driver.
pub const BUILT_AGAINST_RUSTC: &str = env!("CARGO_CGP_BUILT_AGAINST_RUSTC");

/// Whether the arguments request the driver's own version (a bare `--version` or `-V`).
pub fn wants_version(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--version" || arg == "-V")
}

/// The multi-line version record the front-end's preflight parses. The first line names
/// the tool and its version; the two `key: value` lines carry the pinned toolchain and the
/// compiler the driver was built against (whose value contains spaces, hence one field per
/// line rather than one packed line).
pub fn version_string() -> String {
    format!(
        "cargo-cgp-driver {TOOL_VERSION}\n\
         pinned-toolchain: {PINNED_TOOLCHAIN}\n\
         built-against-rustc: {BUILT_AGAINST_RUSTC}"
    )
}
