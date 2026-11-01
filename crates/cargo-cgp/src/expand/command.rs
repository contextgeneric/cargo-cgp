//! Running the wrapped `cargo rustc` that produces an expansion.

use std::io::{self, Write};

use anyhow::Context;

use crate::config::{EXPAND_FLAG, EXPAND_ITEM_FLAG};
use crate::exit::exit_code;
use crate::expand::item::take_item;
use crate::expand::output::{ExpansionDir, read_expansion};
use crate::expand::profile::forwards_profile;
use crate::help::{expand_help_text, is_help_flag};
use crate::launch::{split_forwarded, wrapped_cargo};

/// Expand one target's CGP macros and print the result, returning the exit code to propagate.
///
/// `forwarded_args` are the arguments after `expand`; all but `--item <path>` pass straight through to
/// cargo, so target selection (`--lib`, `--bin`, `-p`, `--features`, …) is cargo's own, while `--item`
/// narrows the expansion to one module or item of the target. It is `cargo rustc`
/// rather than `cargo check` because that is the one subcommand which appends rustc arguments to a
/// *single* target's invocation — which is what scopes expand mode to the crate the user asked
/// about, leaving its dependencies to compile normally.
///
/// A clean expansion stops the compilation yet leaves cargo's status at success, so cargo's code
/// says whether the compiler reported an error, and [`expansion_exit_code`] reports it. An error
/// that lets expansion finish, such as a wiring entry naming nothing, still prints what expanded.
pub fn run_expand(forwarded_args: &[String]) -> anyhow::Result<i32> {
    // Answer a help request here rather than forwarding it: `cargo rustc --help` would describe
    // cargo's options and never mention `--item`, the one flag that is ours.
    if asks_for_help(forwarded_args) {
        println!("{}", expand_help_text());
        return Ok(0);
    }

    let (cargo_args, item) = take_item(forwarded_args)?;

    // A directory of this run's own, so nobody else can redirect the driver's write, and no
    // previous run's output can be printed. It is removed when it goes out of scope.
    let directory =
        ExpansionDir::create().context("failed to create a private directory for the expansion")?;
    let output = directory.output_path();

    // Expansion needs no codegen, so build under the `check` profile unless the caller chose a
    // profile of their own.
    let mut tool_cargo_args = Vec::new();
    if !forwards_profile(&cargo_args) {
        tool_cargo_args.extend(["--profile".to_owned(), "check".to_owned()]);
    }

    // Everything after `--` reaches the selected target's rustc invocation, where the driver
    // recognizes the flags and takes over — see `cargo-cgp-driver`'s `expand` module. They follow
    // any rustc arguments of the user's own, after the same `--`.
    let mut tool_rustc_args = vec![format!("{EXPAND_FLAG}={}", output.display())];
    if let Some(item) = &item {
        tool_rustc_args.push(format!("{EXPAND_ITEM_FLAG}={item}"));
    }

    let mut command = wrapped_cargo("rustc", &cargo_args, &tool_cargo_args, &tool_rustc_args)?;

    let status = command
        .status()
        .context("failed to run `cargo rustc` (is cargo on PATH?)")?;

    let expansion = read_expansion(&output);
    match &expansion {
        Some(expansion) => {
            write_expansion(&mut io::stdout(), expansion)
                .context("failed to write the expansion to stdout")?;
        }
        None => {
            // Whatever went wrong, the layer that knows has already said so above — cargo when it
            // would not run the expansion at all (a package with several targets needs `--lib` or
            // `--bin NAME`), the compiler when the crate failed before expanding, the driver when an
            // `--item` path matched nothing. So this line only reports that nothing came back;
            // guessing at a cause here is what made it confusing.
            eprintln!("error: no expansion was produced; see the errors above");
        }
    }
    Ok(expansion_exit_code(exit_code(status), expansion.is_some()))
}

/// Write the expansion to `out`. A reader that closed the pipe, as `head` does once it has enough,
/// got what it wanted, so that is not a failure; any other write error is returned.
pub fn write_expansion(out: &mut impl Write, expansion: &str) -> io::Result<()> {
    match out
        .write_all(expansion.as_bytes())
        .and_then(|()| out.flush())
    {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result,
    }
}

/// The exit code of an expansion, from cargo's code and whether an expansion came back.
///
/// Postcondition: a failure in cargo's code is kept either way, since the compiler reported an error
/// even when it printed what expanded; and with no expansion the result is never 0, since nothing to
/// print is a failure whatever cargo's status says (the driver's `--item` miss leaves cargo at 0).
pub fn expansion_exit_code(cargo_code: i32, produced: bool) -> i32 {
    match (cargo_code, produced) {
        (0, false) => 1,
        (code, _) => code,
    }
}

/// Whether the forwarded arguments ask for `expand`'s help: a help flag cargo would read, not one
/// after a `--`, which goes to rustc.
pub fn asks_for_help(forwarded_args: &[String]) -> bool {
    split_forwarded(forwarded_args)
        .0
        .iter()
        .any(|arg| is_help_flag(arg))
}
