//! The subcommand dispatcher — the entrypoint the `cargo-cgp` binary calls.

use std::env;

use anyhow::bail;

use crate::args::strip_subcommand;
use crate::check::run_check;
use crate::config::CARGO_SUBCOMMAND;

/// Parse the process arguments and dispatch to the matching subcommand, returning the
/// exit code to propagate. Currently only `check` is implemented; it forwards to
/// `cargo check` through the cargo-cgp driver.
pub fn run() -> anyhow::Result<i32> {
    let args = strip_subcommand(env::args(), CARGO_SUBCOMMAND);
    dispatch(&args)
}

/// Route already-normalized arguments (`["check", ...]`) to a subcommand handler. Split
/// out from [`run`] so dispatch can be tested without touching the real environment.
pub fn dispatch(args: &[String]) -> anyhow::Result<i32> {
    match args.split_first() {
        Some((subcommand, rest)) if subcommand == "check" => run_check(rest),
        Some((subcommand, _)) => {
            bail!("unknown cargo-cgp subcommand `{subcommand}` (expected `check`)")
        }
        None => bail!("missing subcommand (expected `cargo-cgp check`)"),
    }
}
