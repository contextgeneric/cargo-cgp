//! Tests for the preflight's pure logic — parsing the driver's `--version` output
//! ([`parse_driver_version`]) and the match decision ([`evaluate`]) — plus the guarantee
//! that the baked-in [`PINNED_TOOLCHAIN`] tracks `rust-toolchain.toml`.

use std::path::Path;

use cargo_cgp::check::{DriverVersion, evaluate, parse_driver_version};
use cargo_cgp::config::PINNED_TOOLCHAIN;

/// The exact line format the driver prints (mirrors `cargo-cgp-driver`'s `version_string`).
const DRIVER_OUTPUT: &str = "cargo-cgp-driver 0.1.0\n\
     pinned-toolchain: nightly-2026-07-02\n\
     built-against-rustc: rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)";

fn sample() -> DriverVersion {
    parse_driver_version(DRIVER_OUTPUT).expect("driver output should parse")
}

#[test]
fn parses_driver_version() {
    let parsed = sample();
    assert_eq!(parsed.tool_version, "0.1.0");
    assert_eq!(parsed.pinned_toolchain, "nightly-2026-07-02");
    assert_eq!(
        parsed.built_against_rustc,
        "rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)"
    );
}

#[test]
fn rejects_foreign_first_line() {
    assert!(
        parse_driver_version("rustc 1.98.0\npinned-toolchain: x\nbuilt-against-rustc: y").is_none()
    );
}

#[test]
fn rejects_missing_fields() {
    assert!(parse_driver_version("cargo-cgp-driver 0.1.0\npinned-toolchain: x").is_none());
}

#[test]
fn evaluate_accepts_a_matching_driver() {
    let driver = sample();
    let installed = "rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)";
    assert!(evaluate("0.1.0", &driver, installed).is_ok());
}

#[test]
fn evaluate_rejects_a_version_mismatch() {
    let driver = sample();
    let installed = "rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)";
    // Front-end is 0.2.0 but the driver reports 0.1.0 — out of lockstep.
    assert!(evaluate("0.2.0", &driver, installed).is_err());
}

#[test]
fn evaluate_rejects_a_rustc_mismatch() {
    let driver = sample();
    // The installed pinned toolchain is a different nightly than the driver was built with.
    let installed = "rustc 1.98.0-nightly (0000000 2026-08-01)";
    assert!(evaluate("0.1.0", &driver, installed).is_err());
}

#[test]
fn pinned_toolchain_matches_the_toolchain_file() {
    let toolchain_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rust-toolchain.toml");
    let text = std::fs::read_to_string(&toolchain_file).expect("reading rust-toolchain.toml");
    let channel = text
        .lines()
        .find_map(|line| {
            let line = line.split('#').next().unwrap_or("").trim();
            let rest = line
                .strip_prefix("channel")?
                .trim_start()
                .strip_prefix('=')?;
            rest.trim().strip_prefix('"')?.split('"').next()
        })
        .expect("a channel in rust-toolchain.toml");
    assert_eq!(PINNED_TOOLCHAIN, channel);
}
