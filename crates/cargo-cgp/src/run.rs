//! The subcommand dispatcher — the entrypoint the `cargo-cgp` binary calls.

use std::env;

use anyhow::bail;

use crate::args::strip_subcommand;
use crate::check::run_check;
use crate::config::CARGO_SUBCOMMAND;
use crate::setup::run_setup;
use crate::update::run_update;

/// Parse the process arguments and dispatch to the matching subcommand, returning the
/// exit code to propagate. `check` forwards to `cargo check` through the cargo-cgp driver;
/// `setup` provisions the toolchain and driver; `update` upgrades the tool.
pub fn run() -> anyhow::Result<i32> {
    let args = strip_subcommand(env::args(), CARGO_SUBCOMMAND);
    dispatch(&args)
}

/// Route already-normalized arguments (`["check", ...]`) to a subcommand handler. Split
/// out from [`run`] so dispatch can be tested without touching the real environment.
pub fn dispatch(args: &[String]) -> anyhow::Result<i32> {
    match args.split_first() {
        Some((subcommand, rest)) if subcommand == "check" => run_check(rest),
        Some((subcommand, rest)) if subcommand == "setup" => run_setup(rest),
        Some((subcommand, rest)) if subcommand == "update" => run_update(rest),
        Some((subcommand, _)) => {
            bail!(
                "unknown cargo-cgp subcommand `{subcommand}` (expected `check`, `setup`, or `update`)"
            )
        }
        None => bail!("missing subcommand (expected `cargo-cgp check`, `setup`, or `update`)"),
    }
}
