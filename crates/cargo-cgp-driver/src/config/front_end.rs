//! The names the driver shares with the front-end, and where a crash is reported.

/// Environment variable through which `cargo-cgp` hands us the active toolchain's
/// sysroot. It is the counterpart of `cargo_cgp::config::SYSROOT_ENV`; the two crates
/// declare it independently and the shared string is the contract between them.
pub const SYSROOT_ENV: &str = "CARGO_CGP_SYSROOT";

/// The rustc flag that sets the sysroot. We inject it (with the value from
/// [`SYSROOT_ENV`]) only when cargo has not already passed one, because rustc cannot
/// infer a sysroot from the driver's out-of-tree location.
pub const SYSROOT_FLAG: &str = "--sysroot";

/// Where a crash in the driver is reported: the ICE report rustc prints for a compiler panic names
/// it, and so does the note on a diagnostic the emitter shows untransformed after a panic of its
/// own.
pub const BUG_REPORT_URL: &str = "https://github.com/contextgeneric/cargo-cgp/issues";

/// The flag through which `cargo cgp expand` puts the driver in expand mode, carrying the path
/// the expansion is written to (`--cgp-expand=<path>`).
///
/// It is the counterpart of `cargo_cgp::config::EXPAND_FLAG`; the two crates declare it
/// independently and the shared string is the contract between them. It is not a rustc flag, so
/// [`crate::args::rustc_args`] strips it before the compiler sees the argument vector.
pub const EXPAND_FLAG: &str = "--cgp-expand";

/// The flag through which `cargo cgp expand --item <path>` narrows the expansion to one module or
/// item (`--cgp-expand-item=<path>`).
///
/// The counterpart of `cargo_cgp::config::EXPAND_ITEM_FLAG`, and stripped from the argument vector
/// the same way [`EXPAND_FLAG`] is.
pub const EXPAND_ITEM_FLAG: &str = "--cgp-expand-item";
