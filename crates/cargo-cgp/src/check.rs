//! Running the wrapped `cargo check`.

use anyhow::Context;

use crate::exit::exit_code;
use crate::launch::wrapped_cargo;

/// Run `cargo check` with the driver wired in as the workspace rustc wrapper, and return the exit
/// code to propagate; the driver transforms every diagnostic, so the front-end only launches cargo.
/// See `cgp-knowledge-base/cargo-cgp/implementation/executable-structure.md`.
pub fn run_check(forwarded_args: &[String]) -> anyhow::Result<i32> {
    let mut command = wrapped_cargo("check", forwarded_args, &[], &[])?;

    // Let cargo inherit our stdio so its output streams straight through untouched.
    let status = command
        .status()
        .context("failed to run `cargo check` (is cargo on PATH?)")?;

    Ok(exit_code(status))
}
