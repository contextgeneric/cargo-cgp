//! Normalizing the process arguments, so `cargo cgp check …` and `cargo-cgp check …` both reach the
//! router as `["check", …]`; see
//! `cgp-knowledge-base/cargo-cgp/implementation/executable-structure.md`.

use std::ffi::OsString;

/// Drop the program name and a leading cargo-inserted `subcommand` token, so the slice starts at
/// the user's own subcommand; only a leading occurrence is dropped.
pub fn strip_subcommand(
    raw_args: impl IntoIterator<Item = String>,
    subcommand: &str,
) -> Vec<String> {
    let mut args = raw_args.into_iter();

    // Drop argv[0], the path to the cargo-cgp binary itself.
    let _program = args.next();

    let mut rest: Vec<String> = args.collect();
    if rest.first().map(String::as_str) == Some(subcommand) {
        rest.remove(0);
    }

    rest
}

/// The process arguments as strings, or an error naming the first that is not valid Unicode.
///
/// `std::env::args` panics on such an argument; this reads them from `args_os` and reports it
/// instead, as rustc's own `raw_args` does. Every argument the front-end forwards passes through
/// its string handling, so it cannot carry one along.
pub fn utf8_args(raw_args: impl IntoIterator<Item = OsString>) -> anyhow::Result<Vec<String>> {
    raw_args
        .into_iter()
        .enumerate()
        .map(|(index, arg)| {
            arg.into_string()
                .map_err(|arg| anyhow::anyhow!("argument {index} is not valid Unicode: {arg:?}"))
        })
        .collect()
}
