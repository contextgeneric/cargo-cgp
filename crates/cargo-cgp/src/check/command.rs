//! Building and running the wrapped `cargo check`.

use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Context;
use cargo_cgp_error_processing::process_cgp_errors;

use crate::check::{driver_path, emit_rendered, parse_cargo_output, sysroot};
use crate::config::{DRIVER_BIN, MESSAGE_FORMAT_ARG, MESSAGE_FORMAT_FLAG, SYSROOT_ENV};

/// Name of the OS variable that lists directories searched for dynamic libraries. The
/// driver links `librustc_driver` from the sysroot, so that directory must be on this
/// path when cargo spawns the driver.
#[cfg(target_os = "macos")]
const DYLIB_PATH_VAR: &str = "DYLD_FALLBACK_LIBRARY_PATH";
#[cfg(target_os = "windows")]
const DYLIB_PATH_VAR: &str = "PATH";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const DYLIB_PATH_VAR: &str = "LD_LIBRARY_PATH";

/// Run `cargo check`, wiring the cargo-cgp driver in as the workspace rustc wrapper, then
/// post-process the diagnostics it produced.
///
/// `forwarded_args` are the arguments after `check`; they are passed straight through to
/// `cargo check`. The front-end adds `--message-format=json` so cargo's diagnostics
/// arrive as a structured stream, captures that stream, runs it through
/// [`process_cgp_errors`], and re-emits the result. Today the processing stage is a
/// pass-through, so the printed output matches what rustc itself would have rendered; as
/// the stage learns to transform CGP errors, only this output changes. The exit code of
/// the `cargo` process is returned so the caller can propagate it.
///
/// Because the diagnostics are processed as a set, they are captured whole rather than
/// streamed: the processed diagnostics are printed first, then cargo's own captured
/// output (progress and the end-of-build summary), preserving the "diagnostics then
/// summary" order rustc's streaming output would have produced. One consequence is that
/// cargo's progress is buffered to the end of the build rather than shown live — the cost
/// of a stage that must see every diagnostic before it can reorder them.
pub fn run_check(forwarded_args: &[String]) -> anyhow::Result<i32> {
    // The compiler cargo would otherwise use — the one whose sysroot the driver needs.
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());

    let driver = driver_path(DRIVER_BIN)?;
    let sysroot = sysroot(&rustc)?;

    let mut command = Command::new("cargo");
    command.arg("check").args(forwarded_args);

    // Capture diagnostics as JSON — unless the caller already chose a message format, in
    // which case honor theirs and leave the raw output untouched.
    let capture = !forwarded_args
        .iter()
        .any(|arg| arg.starts_with(MESSAGE_FORMAT_FLAG));
    if capture {
        command.arg(MESSAGE_FORMAT_ARG);
    }

    // Route only workspace crates through the driver (dependencies keep using plain
    // rustc), exactly as `cargo clippy` does with `clippy-driver`.
    command.env("RUSTC_WORKSPACE_WRAPPER", &driver);
    command.env(SYSROOT_ENV, &sysroot);
    prepend_dylib_path(&mut command, &sysroot.join("lib"));

    // Capture both streams so the processed diagnostics can be printed ahead of cargo's
    // own output regardless of when cargo wrote it.
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run `cargo check` (is cargo on PATH?)")?;

    emit_output(&output, capture)?;

    Ok(output.status.code().unwrap_or(1))
}

/// Emit the captured `cargo check` output. When the front-end captured JSON, parse it,
/// run the diagnostics through [`process_cgp_errors`], print the processed diagnostics,
/// then replay cargo's own output. When the caller chose their own message format,
/// nothing was captured to process, so both streams are replayed verbatim.
fn emit_output(output: &std::process::Output, capture: bool) -> anyhow::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    if capture {
        let captured = parse_cargo_output(&output.stdout);

        // Forward any non-JSON stdout lines verbatim so nothing cargo wrote is dropped.
        for line in &captured.text_lines {
            writeln!(stdout, "{line}").context("forwarding cargo stdout")?;
        }

        let processed = process_cgp_errors(captured.diagnostics);
        emit_rendered(&mut stderr, &processed).context("writing processed diagnostics")?;
    } else {
        stdout
            .write_all(&output.stdout)
            .context("forwarding cargo stdout")?;
    }

    // Cargo's own messages (progress, the "could not compile" summary) follow the
    // diagnostics, as they would when rustc streams straight to the terminal.
    stderr
        .write_all(&output.stderr)
        .context("forwarding cargo stderr")?;

    stdout.flush().ok();
    stderr.flush().ok();
    Ok(())
}

/// Prepend `dir` to the dynamic-library search path of `command`, preserving any
/// existing value, so the spawned driver can load `librustc_driver` from the sysroot.
fn prepend_dylib_path(command: &mut Command, dir: &Path) {
    let mut entries = vec![OsString::from(dir)];
    if let Some(existing) = env::var_os(DYLIB_PATH_VAR) {
        entries.extend(env::split_paths(&existing).map(OsString::from));
    }

    let joined = env::join_paths(entries).expect("sysroot lib path contains an invalid character");
    command.env(DYLIB_PATH_VAR, joined);
}
