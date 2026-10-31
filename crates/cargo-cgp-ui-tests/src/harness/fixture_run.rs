//! Compiling one fixture, through the tool or plain cargo.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::harness::cargo::cargo;
use crate::paths::cargo_cgp_bin;

/// The result of compiling one fixture: the rendered diagnostics and whether the command
/// succeeded.
pub struct Run {
    /// The command's stderr, holding the rendered diagnostics.
    pub stderr: String,
    /// Whether the command exited successfully (the fixture compiled).
    pub success: bool,
}

impl Run {
    fn from_output(output: &std::process::Output) -> Self {
        Run {
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            success: output.status.success(),
        }
    }
}

/// Compile one fixture through `cargo-cgp check` and return the tool's rendered
/// diagnostics from stderr with its exit status. `-q` suppresses cargo's progress lines,
/// leaving the compiler diagnostics; `--color never` keeps the snapshot free of ANSI escapes.
/// This is the "call cargo-cgp directly" pass.
pub fn run_fixture(harness_crate: &Path, fixture: &Path) -> Run {
    let output = run_cargo_cgp(harness_crate, fixture, &["check", "-q", "--color", "never"]);
    Run::from_output(&output)
}

/// Compile one fixture through `cargo-cgp check --message-format=json` and return cargo's JSON
/// message stream from stdout, the form an editor reads. The caller keeps only the compiler
/// messages; cargo's artifact and build-finished lines are noise here.
pub fn run_fixture_json(harness_crate: &Path, fixture: &Path) -> String {
    let output = run_cargo_cgp(
        harness_crate,
        fixture,
        &["check", "-q", "--message-format=json"],
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Expand one fixture through `cargo cgp expand` and return the expansion from **stdout**.
///
/// Unlike the two diagnostic passes, what is captured is stdout: the expansion is the command's
/// output, and its stderr carries only cargo's progress. The fixture is the worker crate's one
/// binary target, so `cargo rustc` needs no target flag to pick it.
///
/// A fixture whose compilation stops before expansion completes (a malformed macro invocation, or a
/// type defined twice, which rustc treats as fatal once name resolution finishes) produces no
/// expansion at all. The marker returned here is what its snapshot records, which pins that the
/// failure happens before the compiler type-checks the generated code.
pub fn run_fixture_expand(harness_crate: &Path, fixture: &Path) -> String {
    let output = run_cargo_cgp(
        harness_crate,
        fixture,
        &["expand", "-q", "--color", "never"],
    );
    let expansion = String::from_utf8_lossy(&output.stdout).into_owned();

    if expansion.trim().is_empty() {
        return NO_EXPANSION.to_owned();
    }
    expansion
}

/// What an `.expand.rs` snapshot records for a fixture that never got as far as an expansion.
pub const NO_EXPANSION: &str = "// no expansion: compilation stops before macro expansion and name resolution\n\
     // finish, so there is nothing to expand.\n";

/// Compile one fixture through plain `cargo check` — no `cargo-cgp`, no driver — and return
/// its rendered diagnostics from stderr with its exit status. This records the *original* compiler output for the
/// fixture, the "before" against which the tool's `.cgp.stderr` is the "after": no
/// diagnostic rewriting, the default trait solver rather than the driver's
/// `-Znext-solver=globally`, and none of the `--verbose` un-eliding the driver injects.
///
/// It builds in a separate target directory (`target-rust/`) from the `cargo-cgp` passes, so
/// the two do not invalidate each other's `cgp` build: cargo's fingerprint includes
/// `RUSTC_WORKSPACE_WRAPPER`, which `cargo-cgp` sets and plain `cargo` does not, so sharing a
/// target directory would recompile `cgp` on every alternation.
pub fn run_fixture_rust(harness_crate: &Path, fixture: &Path) -> Run {
    fs::copy(fixture, harness_crate.join("src/main.rs"))
        .unwrap_or_else(|e| panic!("copying fixture {}: {e}", fixture.display()));

    let output = Command::new(cargo())
        .current_dir(harness_crate)
        .env("CARGO_TARGET_DIR", harness_crate.join("target-rust"))
        .args(["check", "-q", "--color", "never"])
        .output()
        .expect("running plain cargo check on a fixture");
    Run::from_output(&output)
}

/// Copy the fixture in as the throwaway crate's `src/main.rs` and run `cargo-cgp` with
/// the given arguments. Re-copying bumps the file's mtime, which forces cargo to
/// recompile and re-emit diagnostics even when the same fixture was just built by another
/// pass.
///
/// The fixture's `//@env:` variables are set on the command (see [`crate::directives`]).
///
/// `CARGO_CGP_NO_MANAGE` turns off the front-end's provisioning preflight and toolchain
/// forcing: the harness drives the freshly built driver under the toolchain `cargo test`
/// already selected, so there is nothing to verify or install. Without it, `cargo cgp
/// check` would try to confirm a provisioned driver that the source-tree test loop never
/// installs.
pub fn run_cargo_cgp(harness_crate: &Path, fixture: &Path, args: &[&str]) -> std::process::Output {
    fs::copy(fixture, harness_crate.join("src/main.rs"))
        .unwrap_or_else(|e| panic!("copying fixture {}: {e}", fixture.display()));

    Command::new(cargo_cgp_bin())
        .current_dir(harness_crate)
        .env("CARGO_CGP_NO_MANAGE", "1")
        .envs(crate::directives::read(fixture).env)
        .args(args)
        .output()
        .expect("running cargo-cgp on a fixture")
}
