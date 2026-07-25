//! Running the wrapped `cargo check`.

use anyhow::Context;

use crate::launch::wrapped_cargo;

/// Run `cargo check` with the cargo-cgp driver wired in as the workspace rustc wrapper,
/// and return the exit code so the caller can propagate it.
///
/// `forwarded_args` are the arguments after `check`; they pass straight through to `cargo
/// check`. The front-end does nothing to the diagnostics: the driver's emitter applies
/// every CGP transform in-process and renders the result (text or JSON, matching whatever
/// format the invocation asks for), so the front-end only wires the driver in — through
/// [`wrapped_cargo`], which `expand` shares — and lets cargo's output stream to the terminal
/// untouched.
pub fn run_check(forwarded_args: &[String]) -> anyhow::Result<i32> {
    let mut command = wrapped_cargo("check", forwarded_args)?;

    // Let cargo inherit our stdio so its output streams straight through untouched.
    let status = command
        .status()
        .context("failed to run `cargo check` (is cargo on PATH?)")?;

    Ok(status.code().unwrap_or(1))
}
