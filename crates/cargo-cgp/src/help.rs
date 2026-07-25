//! The `--help` text for the `cargo-cgp` front-end.
//!
//! Shown for `cargo cgp --help` / `-h` and when the tool is run with no subcommand (see
//! [`crate::run::dispatch`]). Kept a pure function that returns the text, so the wording is
//! unit-tested and the actual printing stays at the dispatch edge.

use crate::config::TOOL_VERSION;

/// Whether `arg` is a help flag (`--help` or `-h`).
pub fn is_help_flag(arg: &str) -> bool {
    arg == "--help" || arg == "-h"
}

/// The front-end help text: the tagline, the two invocation forms, the four subcommands,
/// and the top-level options.
pub fn help_text() -> String {
    format!(
        "cargo-cgp {TOOL_VERSION}
Make Context-Generic Programming (CGP) compiler errors readable.

Usage:
    cargo cgp <COMMAND> [ARGS]...
    cargo-cgp <COMMAND> [ARGS]...

Commands:
    check     Check the current package like `cargo check`, presenting CGP errors
              root-cause first. Arguments after `check` are forwarded to `cargo check`.
    expand    Show the Rust one target's CGP macros generate, with CGP's type-level
              constructs resugared. Arguments after `expand` are forwarded to
              `cargo rustc`, so target selection (--lib, --bin, -p) is cargo's own;
              add `--item <path>` to expand just one module or item of it.
    setup     Install the pinned nightly toolchain and build the matching driver.
    update    Upgrade cargo-cgp to the latest published version.

Options:
    -h, --help    Print this help.

Expand options:
    --item <PATH>  Expand only this module or item, as a `::`-separated path — a module
                   (its contents), a type (with the impls for it), or a trait (with the
                   impls of it). Example: `cargo cgp expand --lib --item shapes::Rectangle`.

Run `cargo cgp check --help` to see the underlying `cargo check` options."
    )
}
