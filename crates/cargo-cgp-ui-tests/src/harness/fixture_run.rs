//! Compiling one fixture, through the tool or plain cargo.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::harness::capture::{Captured, capture};
use crate::harness::cargo::cargo;
use crate::options::Threads;
use crate::paths::cargo_cgp_bin;

/// The result of compiling one fixture: the rendered diagnostics and how the command ended.
pub struct Run {
    /// The command's stderr, holding the rendered diagnostics.
    pub stderr: String,
    /// Whether the command exited successfully (the fixture compiled).
    pub success: bool,
    /// Whether the command was killed at its deadline.
    pub timed_out: bool,
}

impl Run {
    fn from_captured(captured: Captured) -> Self {
        Run {
            stderr: captured.stderr,
            success: captured.success,
            timed_out: captured.timed_out,
        }
    }
}

/// Compile one fixture through `cargo-cgp check` and return the tool's rendered
/// diagnostics from stderr with its exit status. `-q` suppresses cargo's progress lines,
/// leaving the compiler diagnostics; `--color never` keeps the snapshot free of ANSI escapes.
/// This is the "call cargo-cgp directly" pass.
pub fn run_fixture(
    harness_crate: &Path,
    fixture: &Path,
    env: &[(String, String)],
    timeout: Duration,
) -> Run {
    let captured = run_cargo_cgp(
        harness_crate,
        fixture,
        env,
        &["check", "-q", "--color", "never"],
        timeout,
    );
    Run::from_captured(captured)
}

/// Compile one fixture through `cargo-cgp check --message-format=json` and return cargo's JSON
/// message stream from stdout, the form an editor reads, and whether the command timed out. The
/// caller keeps only the compiler messages; cargo's artifact and build-finished lines are noise
/// here.
pub fn run_fixture_json(
    harness_crate: &Path,
    fixture: &Path,
    env: &[(String, String)],
    timeout: Duration,
) -> (String, bool) {
    let captured = run_cargo_cgp(
        harness_crate,
        fixture,
        env,
        &["check", "-q", "--message-format=json"],
        timeout,
    );
    (captured.stdout, captured.timed_out)
}

/// One fixture's `cargo cgp expand`: the expansion from **stdout**, and how the command ended.
pub struct ExpandRun {
    /// The expansion, or [`NO_EXPANSION`] when the command printed none.
    pub expansion: String,
    /// How the command ended, for the expand status check.
    pub run: Run,
}

/// Expand one fixture through `cargo cgp expand`.
///
/// Unlike the two diagnostic passes, the snapshot's text is stdout: the expansion is the command's
/// output. The fixture is the worker crate's one binary target, so `cargo rustc` needs no target
/// flag to pick it.
///
/// A fixture whose compilation stops before expansion and name resolution finish (a type defined
/// twice, a `macro_rules!` call matching no rule, or an unclosed delimiter) produces no expansion
/// at all. Its snapshot records the marker, which pins that the failure happens before the compiler
/// type-checks the generated code.
pub fn run_fixture_expand(
    harness_crate: &Path,
    fixture: &Path,
    env: &[(String, String)],
    timeout: Duration,
) -> ExpandRun {
    let mut captured = run_cargo_cgp(
        harness_crate,
        fixture,
        env,
        &["expand", "-q", "--color", "never"],
        timeout,
    );
    let stdout = std::mem::take(&mut captured.stdout);
    let expansion = if stdout.trim().is_empty() {
        NO_EXPANSION.to_owned()
    } else {
        stdout
    };
    ExpandRun {
        expansion,
        run: Run::from_captured(captured),
    }
}

/// What an `.expand.rs` snapshot records for a fixture that never got as far as an expansion.
pub const NO_EXPANSION: &str = "// no expansion: compilation stops before macro expansion and name resolution\n\
     // finish, so there is nothing to expand.\n";

/// Compile one fixture through plain `cargo check`, with no `cargo-cgp` or driver, and return its
/// rendered diagnostics from stderr with its exit status. This records the *original* compiler
/// output for the fixture, the "before" against which the tool's `.cgp.stderr` is the "after": no
/// diagnostic rewriting, the default trait solver rather than the driver's
/// `-Znext-solver=globally`, and none of the `--verbose` un-eliding the driver injects.
///
/// It runs rustc's front end on one thread, by appending [`BASELINE_THREADS_FLAG`] to the ambient
/// `RUSTFLAGS` ([`baseline_rustflags`]), so the baseline records the order a sequential compiler
/// reports in, whatever thread count the tool's passes run with.
///
/// It builds in a separate target directory (`target-rust/`) from the `cargo-cgp` passes, so
/// the two do not invalidate each other's `cgp` build: cargo's fingerprint includes
/// `RUSTC_WORKSPACE_WRAPPER`, which `cargo-cgp` sets and plain `cargo` does not, so sharing a
/// target directory would recompile `cgp` on every alternation.
pub fn run_fixture_rust(harness_crate: &Path, fixture: &Path, timeout: Duration) -> Run {
    fs::copy(fixture, harness_crate.join("src/main.rs"))
        .unwrap_or_else(|e| panic!("copying fixture {}: {e}", fixture.display()));

    let mut command = Command::new(cargo());
    command
        .current_dir(harness_crate)
        .env("CARGO_TARGET_DIR", harness_crate.join("target-rust"))
        .env(
            "RUSTFLAGS",
            baseline_rustflags(std::env::var("RUSTFLAGS").ok().as_deref()),
        )
        .args(["check", "-q", "--color", "never"]);
    Run::from_captured(capture(&mut command, timeout))
}

/// The flag the plain `cargo check` pass appends to `RUSTFLAGS`, so rustc's front end runs on one
/// thread and its diagnostics keep one order between runs.
pub const BASELINE_THREADS_FLAG: &str = "-Zthreads=1";

/// The `RUSTFLAGS` of the plain `cargo check` pass: the ambient value with
/// [`BASELINE_THREADS_FLAG`] appended, since setting the variable replaces the ambient one. rustc
/// takes the last `-Zthreads`, so the flag also overrides an ambient thread count.
pub fn baseline_rustflags(ambient: Option<&str>) -> String {
    join_flags(ambient, Some(BASELINE_THREADS_FLAG))
}

/// The environment of the tool's passes: the fixture's `//@env:` variables, with `RUSTFLAGS` set
/// to the fixture's value when it declares one, else `ambient`, followed by the thread count
/// `threads` names. With [`Threads::Default`] and no flags at all, `RUSTFLAGS` is left unset.
///
/// The check, JSON, and expand passes take the same value, because `RUSTFLAGS` is part of cargo's
/// fingerprint, and a value that differed between them would rebuild `cgp` at every pass.
///
/// Postcondition: the result holds the fixture's variables other than `RUSTFLAGS`, in their
/// order, then at most one `RUSTFLAGS`, which ends with `threads.flag()` when it is `Some`.
pub fn tool_env(
    fixture_env: &[(String, String)],
    ambient: Option<&str>,
    threads: Threads,
) -> Vec<(String, String)> {
    let declared = fixture_env
        .iter()
        .rev()
        .find(|(key, _)| key == "RUSTFLAGS")
        .map(|(_, value)| value.as_str());
    let rustflags = join_flags(declared.or(ambient), threads.flag().as_deref());

    let mut env: Vec<(String, String)> = fixture_env
        .iter()
        .filter(|(key, _)| key != "RUSTFLAGS")
        .cloned()
        .collect();
    if !rustflags.is_empty() {
        env.push(("RUSTFLAGS".to_owned(), rustflags));
    }
    env
}

/// `base` and `extra` joined by a space, either left out when blank.
fn join_flags(base: Option<&str>, extra: Option<&str>) -> String {
    [base, extra]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|flags| !flags.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Copy the fixture in as the throwaway crate's `src/main.rs` and run `cargo-cgp` with
/// the given arguments. Re-copying bumps the file's mtime, which forces cargo to
/// recompile and re-emit diagnostics even when the same fixture was just built by another
/// pass.
///
/// `env` holds the variables [`tool_env`] gives: the fixture's `//@env:` variables (see
/// [`crate::directives`]) and the `RUSTFLAGS` that carries the thread count.
///
/// `CARGO_CGP_NO_MANAGE` turns off the front-end's provisioning preflight and toolchain
/// forcing: the harness drives the freshly built driver under the toolchain `cargo test`
/// already selected, so there is nothing to verify or install. Without it, `cargo cgp
/// check` would try to confirm a provisioned driver that the source-tree test loop never
/// installs.
pub fn run_cargo_cgp(
    harness_crate: &Path,
    fixture: &Path,
    env: &[(String, String)],
    args: &[&str],
    timeout: Duration,
) -> Captured {
    fs::copy(fixture, harness_crate.join("src/main.rs"))
        .unwrap_or_else(|e| panic!("copying fixture {}: {e}", fixture.display()));

    let mut command = Command::new(cargo_cgp_bin());
    command
        .current_dir(harness_crate)
        .env("CARGO_CGP_NO_MANAGE", "1")
        .envs(env.iter().map(|(key, value)| (key, value)))
        .args(args);
    capture(&mut command, timeout)
}
