//! Building the tool and compiling a fixture through it.
//!
//! A fixture is a loose `.rs` file, so it is compiled by copying it into a throwaway
//! crate that depends on cgp and running `cargo-cgp check` there. Reusing one crate
//! keeps cgp built and cached across fixtures.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::paths::{cargo_cgp_bin, cgp_crate_dir};

/// The cargo executable, from the `CARGO` cargo sets for us, falling back to `cargo`.
fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

/// Build the `cargo-cgp` front-end and its driver into the workspace `target/debug`.
/// Both are needed: the front-end locates the driver as its sibling.
pub fn build_binaries() {
    let status = Command::new(cargo())
        .current_dir(workspace_root_for_build())
        .args([
            "build",
            "-q",
            "--bin",
            "cargo-cgp",
            "--bin",
            "cargo-cgp-driver",
        ])
        .status()
        .expect("running `cargo build` for the cargo-cgp binaries");
    assert!(status.success(), "failed to build the cargo-cgp binaries");
}

/// Where `cargo build` runs. The workspace root holds `rust-toolchain.toml`, so
/// building there selects the pinned toolchain — the one whose compiler the driver
/// embeds.
fn workspace_root_for_build() -> PathBuf {
    crate::paths::workspace_root()
}

/// Create or refresh the throwaway crate the fixtures are compiled in, and return its
/// directory. It lives under `target/` (git-ignored). Naming it `ui` keeps cargo's
/// output stable; the empty `[workspace]` table stops cargo from folding it into the
/// cargo-cgp workspace above it in `target/`.
pub fn ensure_harness_crate() -> PathBuf {
    let dir = crate::paths::harness_crate_dir();
    fs::create_dir_all(dir.join("src")).expect("creating the throwaway crate");

    let manifest = format!(
        "[package]\n\
         name    = \"ui\"\n\
         version = \"0.0.0\"\n\
         edition = \"2024\"\n\
         publish = false\n\
         \n\
         [dependencies]\n\
         cgp = {{ path = \"{}\" }}\n\
         \n\
         [workspace]\n",
        cgp_crate_dir().display(),
    );
    fs::write(dir.join("Cargo.toml"), manifest).expect("writing the throwaway manifest");

    let main = dir.join("src/main.rs");
    if !main.exists() {
        fs::write(&main, "fn main() {}\n").expect("seeding the throwaway main.rs");
    }

    dir
}

/// Compile one fixture through `cargo-cgp check` and return the tool's rendered
/// diagnostics from stderr. `-q` suppresses cargo's progress lines, leaving the compiler
/// diagnostics; `--color never` keeps the snapshot free of ANSI escapes. This is the
/// "call cargo-cgp directly" pass.
pub fn run_fixture(harness_crate: &Path, fixture: &Path) -> String {
    let output = run_cargo_cgp(harness_crate, fixture, &["check", "-q", "--color", "never"]);
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Compile one fixture through `cargo-cgp check --message-format=json` and return the raw
/// JSON stream from stdout. With a message format already set, the front-end forwards
/// cargo's JSON unchanged rather than processing it — so this is exactly the diagnostic
/// stream the tool captures internally, which the caller parses into the diagnostics fed
/// to `process_cgp_errors`.
pub fn run_fixture_json(harness_crate: &Path, fixture: &Path) -> Vec<u8> {
    let output = run_cargo_cgp(
        harness_crate,
        fixture,
        &["check", "-q", "--color", "never", "--message-format=json"],
    );
    output.stdout
}

/// Copy the fixture in as the throwaway crate's `src/main.rs` and run `cargo-cgp` with
/// the given arguments. Re-copying bumps the file's mtime, which forces cargo to
/// recompile and re-emit diagnostics even when the same fixture was just built by another
/// pass.
fn run_cargo_cgp(harness_crate: &Path, fixture: &Path, args: &[&str]) -> std::process::Output {
    fs::copy(fixture, harness_crate.join("src/main.rs"))
        .unwrap_or_else(|e| panic!("copying fixture {}: {e}", fixture.display()));

    Command::new(cargo_cgp_bin())
        .current_dir(harness_crate)
        .args(args)
        .output()
        .expect("running cargo-cgp on a fixture")
}
