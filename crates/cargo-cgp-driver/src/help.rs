//! The driver's own `--help` text.
//!
//! When `cargo-cgp-driver` is run directly (not as cargo's rustc wrapper), a `--help` or
//! `-h` flag — and a bare invocation with no arguments — prints this and exits, instead of
//! running the compiler. In *wrapper* mode the same flags belong to the real compiler, so
//! [`crate::run`] answers here only when not in wrapper mode, exactly as it does for
//! [`crate::version`].

use crate::version::{PINNED_TOOLCHAIN, TOOL_VERSION};

/// Whether the arguments request help (a bare `--help` or `-h`).
pub fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--help" || arg == "-h")
}

/// The driver's help text. Since the driver is normally invoked by cargo rather than by a
/// person, the text explains that role and the few flags it answers when run directly.
pub fn help_text() -> String {
    format!(
        "cargo-cgp-driver {TOOL_VERSION}
The rustc wrapper behind cargo-cgp — not meant to be run directly.

cargo runs this in place of rustc for each workspace crate (through
RUSTC_WORKSPACE_WRAPPER), passing the real rustc path followed by its arguments.
In that wrapper mode every argument is forwarded to the compiler.

Run directly, it answers:
    -h, --help       Print this help.
    -V, --version    Print the driver version, its pinned toolchain
                     ({PINNED_TOOLCHAIN}), and the rustc it was built against.

To check a project, use `cargo cgp check`. To drive the compiler by hand for
debugging, see the cargo-cgp usage reference."
    )
}
