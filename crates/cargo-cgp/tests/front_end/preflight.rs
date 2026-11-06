//! Tests for the preflight's pure logic: parsing the driver's `--version` output
//! ([`parse_driver_version`]) and the match decision ([`evaluate`]), plus the guarantee
//! that the baked-in [`PINNED_TOOLCHAIN`] tracks `rust-toolchain.toml`.

use std::path::Path;

use cargo_cgp::config::PINNED_TOOLCHAIN;
use cargo_cgp::launch::{
    DriverLocation, DriverOrigin, DriverVersion, PreflightFailure, evaluate, failure_message,
    judge_driver_answer, parse_driver_version, toolchain_failure,
};
use cargo_cgp::toolchain::ToolchainError;

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

// Property: the baked-in `PINNED_TOOLCHAIN` is the channel `rust-toolchain.toml` pins: the file
//   holds exactly one line whose words are `channel`, `=`, and the quoted toolchain. The oracle
//   splits lines on whitespace and shares no code with `build.rs`'s `parse_channel`, so a fault in
//   that parser cannot make both sides agree.
#[test]
fn pinned_toolchain_matches_the_toolchain_file() {
    let toolchain_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rust-toolchain.toml");
    let text = std::fs::read_to_string(&toolchain_file).expect("reading rust-toolchain.toml");
    let quoted = format!("\"{PINNED_TOOLCHAIN}\"");
    let expected = ["channel", "=", quoted.as_str()];
    let pinning_lines = text
        .lines()
        .filter(|line| line.split_whitespace().eq(expected))
        .count();
    assert_eq!(
        pinning_lines, 1,
        "rust-toolchain.toml should pin {quoted}:\n{text}"
    );
}

fn sibling_driver() -> DriverLocation {
    DriverLocation {
        path: "/home/u/.cargo/bin/cargo-cgp-driver".into(),
        origin: DriverOrigin::Sibling,
    }
}

// Property: `evaluate` reports `RustcMismatch` when only the compilers differ, and
//   `VersionMismatch` when the versions differ, each naming the values that differ.
// Property (assumed from the check order, since the doc does not rank the two):
//   `VersionMismatch` is the one reported when the compilers differ as well.
#[test]
fn evaluate_reports_which_identity_differs() {
    let driver = sample();
    let matching = "rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)";
    let installed = "rustc 1.98.0-nightly (0000000 2026-08-01)";
    for installed_rustc in [matching, installed] {
        assert_eq!(
            evaluate("0.2.0", &driver, installed_rustc),
            Err(PreflightFailure::VersionMismatch {
                driver_version: "0.1.0".to_owned()
            })
        );
    }
    assert_eq!(
        evaluate("0.1.0", &driver, installed),
        Err(PreflightFailure::RustcMismatch {
            built_against: "rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)".to_owned(),
            installed: installed.to_owned(),
        })
    );
}

#[test]
fn a_rustc_mismatch_names_the_toolchain_that_was_queried() {
    // The driver says it is pinned to `nightly-2026-07-02`, but the compiler it is compared against
    // came from the toolchain the preflight queried, which an override can make another one.
    let failure = PreflightFailure::RustcMismatch {
        built_against: "rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)".to_owned(),
        installed: "rustc 1.99.0-nightly (1111111 2026-08-01)".to_owned(),
    };
    let message = failure_message(&failure, "nightly-2026-08-02", &sibling_driver(), "0.1.0");

    assert!(
        message.contains("toolchain `nightly-2026-08-02` provides `rustc 1.99.0-nightly"),
        "{message}"
    );
    assert!(!message.contains("nightly-2026-07-02"), "{message}");
}

#[test]
fn a_driver_that_fails_to_load_keeps_the_loaders_reason() {
    let failure = PreflightFailure::DriverDidNotRun {
        stderr: "cargo-cgp-driver: error while loading shared libraries: librustc_driver-abc.so: \
                 cannot open shared object file\n"
            .to_owned(),
    };
    let message = failure_message(&failure, "nightly-2026-10-10", &sibling_driver(), "0.1.0");

    assert!(message.contains("librustc_driver-abc.so"), "{message}");
    assert!(
        message.ends_with("help: run `cargo cgp setup`"),
        "{message}"
    );
}

#[test]
fn every_driver_message_names_the_driver_and_where_it_was_found() {
    let on_path = DriverLocation {
        path: "/nix/profile/bin/cargo-cgp-driver".into(),
        origin: DriverOrigin::Path,
    };
    for failure in [
        PreflightFailure::DriverDidNotStart {
            error: "Permission denied (os error 13)".to_owned(),
        },
        PreflightFailure::DriverDidNotRun {
            stderr: String::new(),
        },
        PreflightFailure::UnparsableVersion,
        PreflightFailure::VersionMismatch {
            driver_version: "0.0.9".to_owned(),
        },
        PreflightFailure::RustcMismatch {
            built_against: "a".to_owned(),
            installed: "b".to_owned(),
        },
    ] {
        let message = failure_message(&failure, "nightly-2026-10-10", &on_path, "0.1.0");
        assert!(
            message.contains("/nix/profile/bin/cargo-cgp-driver")
                && message.contains("found on PATH, since none is beside cargo-cgp"),
            "{message}"
        );
        assert!(
            message.ends_with("help: run `cargo cgp setup`"),
            "{message}"
        );
    }
}

#[test]
fn a_missing_toolchain_keeps_rustups_reason() {
    let failure = PreflightFailure::ToolchainUnavailable {
        stderr: "error: toolchain 'nightly-2026-10-10' is not installed\n".to_owned(),
    };
    let message = failure_message(&failure, "nightly-2026-10-10", &sibling_driver(), "0.1.0");

    assert_eq!(
        message,
        "toolchain `nightly-2026-10-10` is not available:\n\n  \
         error: toolchain 'nightly-2026-10-10' is not installed\n\n\
         help: run `cargo cgp setup`"
    );
}

#[test]
fn a_missing_rustup_says_so_instead_of_naming_setup() {
    // `setup` needs rustup too, so naming it would send the user round in a circle.
    let message = failure_message(
        &PreflightFailure::RustupMissing,
        "nightly-2026-10-10",
        &sibling_driver(),
        "0.1.0",
    );

    assert!(
        message.contains("rustup was not found on PATH"),
        "{message}"
    );
    assert!(
        message.contains("help: install rustup, or install cargo-cgp through its Nix flake"),
        "{message}"
    );
    assert!(!message.contains("cargo cgp setup"), "{message}");
}

#[test]
fn a_driver_that_failed_keeps_its_stderr() {
    let verdict = judge_driver_answer(
        false,
        b"",
        b"cannot open shared object file\n",
        "0.1.0",
        "x",
    );
    assert_eq!(
        verdict,
        Err(PreflightFailure::DriverDidNotRun {
            stderr: "cannot open shared object file\n".to_owned()
        })
    );
}

#[test]
fn a_driver_answer_in_another_shape_is_unparsable() {
    let verdict = judge_driver_answer(true, b"rustc 1.98.0\n", b"", "0.1.0", "x");
    assert_eq!(verdict, Err(PreflightFailure::UnparsableVersion));
}

#[test]
fn a_driver_answer_is_compared_with_the_front_end_and_the_toolchain() {
    let installed = "rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)";
    assert_eq!(
        judge_driver_answer(true, DRIVER_OUTPUT.as_bytes(), b"", "0.1.0", installed),
        Ok(())
    );
    assert_eq!(
        judge_driver_answer(true, DRIVER_OUTPUT.as_bytes(), b"", "0.2.0", installed),
        Err(PreflightFailure::VersionMismatch {
            driver_version: "0.1.0".to_owned()
        })
    );
}

#[test]
fn a_toolchain_query_failure_maps_to_its_preflight_failure() {
    assert_eq!(
        toolchain_failure(ToolchainError::RustupMissing).unwrap(),
        PreflightFailure::RustupMissing
    );
    assert_eq!(
        toolchain_failure(ToolchainError::QueryFailed {
            stderr: "error: toolchain 'x' is not installed".to_owned()
        })
        .unwrap(),
        PreflightFailure::ToolchainUnavailable {
            stderr: "error: toolchain 'x' is not installed".to_owned()
        }
    );
    // Any other failure has no `setup` to suggest, so it passes through as an error.
    assert!(toolchain_failure(ToolchainError::Other(anyhow::anyhow!("unreadable"))).is_err());
}
