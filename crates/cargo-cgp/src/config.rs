//! Well-known names shared across the front-end.
//!
//! These are the few string constants the subcommand cannot avoid hardcoding — the
//! cargo subcommand name, the driver executable name, and the environment variable
//! used to hand the sysroot to the driver. They live in one place so the functions
//! that use them take them as parameters rather than embedding literals, keeping the
//! call sites loosely coupled.

/// The name cargo passes as the first argument when the binary is invoked as
/// `cargo cgp <...>`. Cargo runs `cargo-cgp cgp <...>`, so the leading `cgp` is
/// stripped by [`crate::args::strip_subcommand`] to make the same entrypoint work
/// when the binary is run directly as `cargo-cgp <...>`.
pub const CARGO_SUBCOMMAND: &str = "cgp";

/// Executable name of the rustc wrapper cargo-cgp launches. The driver is expected to
/// sit next to the `cargo-cgp` binary in the same directory (see
/// [`crate::launch::driver_path`]).
pub const DRIVER_BIN: &str = "cargo-cgp-driver";

/// Environment variable through which cargo-cgp hands the active toolchain's sysroot
/// to the driver. The driver reads it to inject `--sysroot` when cargo does not.
/// The driver crate declares the same name independently; the two form a contract.
pub const SYSROOT_ENV: &str = "CARGO_CGP_SYSROOT";

/// The exact nightly the driver is welded to, baked in from the workspace
/// `rust-toolchain.toml` by `build.rs`. `cargo cgp check` forces this toolchain for the
/// wrapped compilation, and `setup`/`update` install and build against it. It can be
/// overridden at runtime with [`TOOLCHAIN_ENV`] (see [`crate::toolchain::pinned_toolchain`]).
pub const PINNED_TOOLCHAIN: &str = env!("CARGO_CGP_PINNED_TOOLCHAIN");

/// The rustup environment variable that selects a toolchain. cargo-cgp sets it to the
/// pinned nightly on the wrapped `cargo check` so the sysroot and `librustc_driver` match
/// the compiler the driver embeds; it outranks a project's `rust-toolchain.toml`.
pub const RUSTUP_TOOLCHAIN_ENV: &str = "RUSTUP_TOOLCHAIN";

/// Escape hatch: an explicit path to the driver executable, bypassing the sibling lookup.
/// The UI test harness sets it to the freshly built `target/debug/cargo-cgp-driver`.
pub const DRIVER_ENV: &str = "CARGO_CGP_DRIVER";

/// Escape hatch: when set (to any value), `cargo cgp check` skips the preflight and the
/// toolchain forcing and trusts whatever driver and toolchain the environment provides.
/// Used for local development inside the source checkout under the pinned toolchain.
pub const NO_MANAGE_ENV: &str = "CARGO_CGP_NO_MANAGE";

/// Escape hatch: overrides [`PINNED_TOOLCHAIN`] at runtime, for testing a toolchain bump
/// before the constant changes. Normally paired with [`NO_MANAGE_ENV`], since a hand-picked
/// toolchain will not match the driver's baked-in build identity.
pub const TOOLCHAIN_ENV: &str = "CARGO_CGP_TOOLCHAIN";

/// The default target directory `cargo cgp check` builds into, relative to the working
/// directory, instead of the project's `target/`. Isolating the check's artifacts (built
/// under a forced nightly with extra flags) keeps it from contending with the project's
/// normal builds and with Rust Analyzer. Overridable with an explicit `--target-dir` or
/// `CARGO_TARGET_DIR`.
pub const CHECK_TARGET_DIR: &str = "target/cgp";

/// The rustc flag through which `cargo cgp expand` puts the driver in expand mode, carrying the
/// path the driver writes the finished expansion to (`--cgp-expand=<path>`).
///
/// It is passed after `cargo rustc`'s `--`, so cargo appends it to exactly one target's rustc
/// invocation — which is what scopes expand mode to the crate the user asked about. The driver
/// strips it from the argument vector before the compiler sees it, since it is no flag of rustc's;
/// the driver crate declares the same name independently, and the shared string is the contract
/// between them.
pub const EXPAND_FLAG: &str = "--cgp-expand";

/// The rustc flag through which `cargo cgp expand --item <path>` tells the driver to narrow the
/// expansion to one module or item (`--cgp-expand-item=<path>`).
///
/// A second flag rather than a second value on [`EXPAND_FLAG`], so each carries one thing. Like that
/// flag it is stripped by the driver before the compiler sees the argument vector, and the driver
/// declares the same name independently.
pub const EXPAND_ITEM_FLAG: &str = "--cgp-expand-item";

/// The crate name of the driver on crates.io, installed by `setup` at the front-end's own
/// version to keep the pair in lockstep.
pub const DRIVER_CRATE: &str = "cargo-cgp-driver";

/// The crate name of the front-end on crates.io, reinstalled by `update`.
pub const FRONTEND_CRATE: &str = "cargo-cgp";

/// This front-end's own version, from Cargo. The preflight requires the driver to report
/// the identical version, and `setup` installs the driver at this version.
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
