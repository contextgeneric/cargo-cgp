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

Run `cargo cgp expand --help` for the expand options, and `cargo cgp check --help` to see
the underlying `cargo check` options."
    )
}

/// The `expand` subcommand's help text: what the command shows, how to select a target, and the
/// one option that is cargo-cgp's own rather than cargo's.
///
/// `expand` needs a help text of its own because it is the one subcommand with a flag cargo does not
/// know. Its other arguments go to `cargo rustc`, so forwarding `--help` there — which is what
/// `check` does, usefully — would answer with cargo's help and never mention `--item`.
pub fn expand_help_text() -> String {
    format!(
        "cargo-cgp {TOOL_VERSION}
Show the Rust a target's CGP macros generate, with CGP's type-level constructs resugared:
a field tag reads `Symbol!(\"width\")` rather than a raw `Chars` spine.

Usage:
    cargo cgp expand [OPTIONS] [CARGO ARGS]...

Options:
    --item <PATH>  Expand only this module or item instead of the whole target. The path is
                   `::`-separated and names something inside the crate, with an optional
                   leading `crate::`. What it selects depends on what it names:
                     a module  the module's contents
                     a type    its declaration and every impl written for it
                     a trait   its definition and every impl of it — for a component's
                               provider trait, everything wired to that component
    -h, --help     Print this help.

Every other argument is forwarded to `cargo rustc`, so target selection is cargo's own —
and because exactly one target is expanded, a package with both a library and a binary
needs `--lib` or `--bin NAME`. Run `cargo rustc --help` for those options.

Examples:
    cargo cgp expand --lib
    cargo cgp expand --lib --item contexts::app
    cargo cgp expand --bin server --item AreaCalculator

The expansion is written to stdout, so redirect or pipe it. Note that `expand` is not a
check: it stops once the macros are expanded, so it reports nothing about wiring — use
`cargo cgp check` for that."
    )
}
