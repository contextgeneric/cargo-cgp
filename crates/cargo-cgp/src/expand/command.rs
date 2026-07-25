//! Running the wrapped `cargo rustc` that produces an expansion.

use std::fs;
use std::io::{self, Write};

use anyhow::Context;

use crate::config::{EXPAND_FLAG, EXPAND_ITEM_FLAG};
use crate::expand::item::take_item;
use crate::expand::output::{output_path, read_expansion};
use crate::expand::profile::forwards_profile;
use crate::launch::wrapped_cargo;

/// Expand one target's CGP macros and print the result, returning the exit code to propagate.
///
/// `forwarded_args` are the arguments after `expand`; all but `--item <path>` pass straight through to
/// cargo, so target selection (`--lib`, `--bin`, `-p`, `--features`, …) is cargo's own, while `--item`
/// narrows the expansion to one module or item of the target. It is `cargo rustc`
/// rather than `cargo check` because that is the one subcommand which appends rustc arguments to a
/// *single* target's invocation — which is what scopes expand mode to the crate the user asked
/// about, leaving its dependencies to compile normally.
///
/// Success is judged by whether the driver produced output, not by cargo's exit code: the
/// compilation stops after expansion, so the unit yields no artifact and cargo may report a
/// failure for a run that did exactly what was asked.
pub fn run_expand(forwarded_args: &[String]) -> anyhow::Result<i32> {
    let (cargo_args, item) = take_item(forwarded_args)?;

    let output = output_path();
    // Clear any leftover output, so a failed run can never print a previous expansion.
    let _ = fs::remove_file(&output);

    let mut command = wrapped_cargo("rustc", &cargo_args)?;

    // Expansion needs no codegen, so build under the `check` profile unless the caller chose a
    // profile of their own.
    if !forwards_profile(&cargo_args) {
        command.arg("--profile").arg("check");
    }

    // Everything after `--` reaches the selected target's rustc invocation, where the driver
    // recognizes the flags and takes over — see `cargo-cgp-driver`'s `expand` module.
    command.arg("--");
    command.arg(format!("{EXPAND_FLAG}={}", output.display()));
    if let Some(item) = &item {
        command.arg(format!("{EXPAND_ITEM_FLAG}={item}"));
    }

    let status = command
        .status()
        .context("failed to run `cargo rustc` (is cargo on PATH?)")?;

    match read_expansion(&output) {
        Some(expansion) => {
            let _ = fs::remove_file(&output);
            io::stdout()
                .write_all(expansion.as_bytes())
                .context("failed to write the expansion to stdout")?;
            Ok(0)
        }
        None => {
            let _ = fs::remove_file(&output);
            // With a filter, the likelier cause is a path that names nothing — so say both rather
            // than blame the compilation for what is usually a typo.
            match &item {
                Some(item) => eprintln!(
                    "error: nothing was expanded for `{item}` — no such module or item, or the target did not compile far enough to expand"
                ),
                None => eprintln!(
                    "error: no expansion was produced — the target did not compile far enough to expand"
                ),
            }
            Ok(status.code().filter(|code| *code != 0).unwrap_or(1))
        }
    }
}
