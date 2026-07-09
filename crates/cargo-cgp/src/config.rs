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
/// [`crate::check::driver_path`]).
pub const DRIVER_BIN: &str = "cargo-cgp-driver";

/// Environment variable through which cargo-cgp hands the active toolchain's sysroot
/// to the driver. The driver reads it to inject `--sysroot` when cargo does not.
/// The driver crate declares the same name independently; the two form a contract.
pub const SYSROOT_ENV: &str = "CARGO_CGP_SYSROOT";

/// The `cargo check` flag that switches its output to the machine-readable JSON stream
/// the front-end parses to capture diagnostics. The `rendered` field of each message
/// still holds rustc's own pretty text, so re-emitting it reproduces the human output.
pub const MESSAGE_FORMAT_ARG: &str = "--message-format=json";

/// Prefix of the message-format flag, used to detect a caller who already set one so the
/// front-end does not append a conflicting second `--message-format`.
pub const MESSAGE_FORMAT_FLAG: &str = "--message-format";
