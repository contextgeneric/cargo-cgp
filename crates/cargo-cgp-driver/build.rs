//! Bakes the driver's toolchain identity into it at build time.
//!
//! Two values are recorded. `CARGO_CGP_PINNED_TOOLCHAIN` is the channel the workspace
//! `rust-toolchain.toml` pins — the toolchain the driver is *meant* to be built with.
//! `CARGO_CGP_BUILT_AGAINST_RUSTC` is the `rustc --version` line of the compiler that
//! *actually* compiled the driver, captured by querying the compiling rustc, so the
//! front-end's preflight can tell whether the driver was built against the nightly that is
//! installed rather than merely the one it was supposed to be.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    let toolchain_file = find_toolchain_file();
    println!("cargo::rerun-if-changed={}", toolchain_file.display());
    println!("cargo::rerun-if-env-changed=RUSTC");

    let text = std::fs::read_to_string(&toolchain_file)
        .unwrap_or_else(|e| panic!("reading {}: {e}", toolchain_file.display()));
    let channel = parse_channel(&text)
        .unwrap_or_else(|| panic!("no `channel` in {}", toolchain_file.display()));
    println!("cargo::rustc-env=CARGO_CGP_PINNED_TOOLCHAIN={channel}");

    println!(
        "cargo::rustc-env=CARGO_CGP_BUILT_AGAINST_RUSTC={}",
        built_against_rustc()
    );
}

/// Locate the `rust-toolchain.toml` that pins the channel. A source checkout keeps the one
/// canonical file at the workspace root; a published crate ships its own copy in the crate
/// root (see the crate's `rust-toolchain.toml`), because the workspace file is not part of
/// the package tarball. Walking up from the manifest directory takes the first match, so the
/// same build script serves both layouts — including the isolated verification build cargo
/// runs under `target/package/` during `cargo publish`.
fn find_toolchain_file() -> PathBuf {
    let start = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    for dir in start.ancestors() {
        let candidate = dir.join("rust-toolchain.toml");
        if candidate.exists() {
            return candidate;
        }
    }
    panic!(
        "could not find rust-toolchain.toml at or above {}",
        start.display()
    );
}

/// The single-line `rustc --version` of the compiler building this driver. cargo sets
/// `RUSTC` to that compiler for build scripts; the short version line carries the release,
/// short commit hash, and date, which uniquely identify a dated nightly.
fn built_against_rustc() -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let output = Command::new(&rustc)
        .arg("--version")
        .output()
        .unwrap_or_else(|e| panic!("running `{rustc} --version`: {e}"));
    assert!(output.status.success(), "`{rustc} --version` failed");
    String::from_utf8(output.stdout)
        .expect("`rustc --version` produced non-UTF-8 output")
        .trim()
        .to_owned()
}

/// Extract the `channel = "..."` value from a `rust-toolchain.toml`, ignoring comments.
/// A deliberately minimal scan rather than a full TOML parse, so the build script pulls in
/// no dependency for one well-known field in a file we own.
fn parse_channel(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some(rest) = line.strip_prefix("channel") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let value = rest.trim();
        let value = value.strip_prefix('"')?.split('"').next()?;
        return Some(value.to_owned());
    }
    None
}
