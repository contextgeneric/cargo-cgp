//! The subcommand dispatcher — the entrypoint the `cargo-cgp` binary calls.

use std::env;

use anyhow::bail;

use crate::args::strip_subcommand;
use crate::check::run_check;
use crate::config::CARGO_SUBCOMMAND;
use crate::expand::run_expand;
use crate::help::{
    help_text, is_help_flag, is_version_flag, setup_help_text, update_help_text, version_text,
};
use crate::setup::run_setup;
use crate::update::run_update;

/// Parse the process arguments and dispatch to the matching subcommand, returning the
/// exit code to propagate. `check` forwards to `cargo check` through the cargo-cgp driver;
/// `expand` shows the Rust a target's CGP macros generate; `setup` provisions the toolchain
/// and driver; `update` upgrades the tool.
pub fn run() -> anyhow::Result<i32> {
    let args = strip_subcommand(env::args(), CARGO_SUBCOMMAND);
    dispatch(&args)
}

/// Route already-normalized arguments (`["check", ...]`) to a subcommand handler. Split
/// out from [`run`] so dispatch can be tested without touching the real environment.
///
/// A leading `--help`/`-h`, or no subcommand at all, prints the [`help_text`] and succeeds,
/// so a bare `cargo cgp` is a friendly overview rather than an error; a leading
/// `--version`/`-V` prints the [`version_text`].
///
/// `setup` and `update` take no arguments of their own. Because both install things, they answer
/// `--help` instead of running, and refuse any other argument rather than silently ignoring it.
pub fn dispatch(args: &[String]) -> anyhow::Result<i32> {
    match args.split_first() {
        None => {
            println!("{}", help_text());
            Ok(0)
        }
        Some((flag, _)) if is_help_flag(flag) => {
            println!("{}", help_text());
            Ok(0)
        }
        Some((flag, _)) if is_version_flag(flag) => {
            println!("{}", version_text());
            Ok(0)
        }
        Some((subcommand, rest)) if subcommand == "check" => run_check(rest),
        Some((subcommand, rest)) if subcommand == "expand" => run_expand(rest),
        Some((subcommand, rest)) if subcommand == "setup" => {
            match no_argument_command(subcommand, rest, setup_help_text)? {
                Some(code) => Ok(code),
                None => run_setup(rest),
            }
        }
        Some((subcommand, rest)) if subcommand == "update" => {
            match no_argument_command(subcommand, rest, update_help_text)? {
                Some(code) => Ok(code),
                None => run_update(rest),
            }
        }
        Some((subcommand, _)) => {
            bail!(
                "unknown cargo-cgp subcommand `{subcommand}` (expected `check`, `expand`, `setup`, or `update`)"
            )
        }
    }
}

/// Handle the arguments of a subcommand that takes none. `--help`/`-h` prints its help and returns
/// `Some(0)`, so nothing runs; any other argument is an error naming it; no arguments at all
/// returns `None`, meaning the caller runs the subcommand.
fn no_argument_command(
    subcommand: &str,
    rest: &[String],
    help: fn() -> String,
) -> anyhow::Result<Option<i32>> {
    match rest.first() {
        None => Ok(None),
        Some(arg) if is_help_flag(arg) => {
            println!("{}", help());
            Ok(Some(0))
        }
        Some(arg) => bail!(
            "`cargo cgp {subcommand}` takes no arguments, but was given `{arg}` \
             (run `cargo cgp {subcommand} --help` to see what it does)"
        ),
    }
}
