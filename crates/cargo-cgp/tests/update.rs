//! Tests for `update`'s pure logic — extracting the latest version from `cargo search`
//! output ([`parse_latest_version`]) and the newer-than decision ([`is_newer`]).

use cargo_cgp::update::{is_newer, parse_latest_version};

/// A representative `cargo search cargo-cgp` output: the exact crate plus lookalikes.
const SEARCH_OUTPUT: &str = "\
cargo-cgp-driver = \"0.3.0\"    # A rustc wrapper...
cargo-cgp = \"0.2.0\"           # A cargo subcommand...
cargo-cgp-error-processing = \"0.3.0\"  # Compiler-free...
... and 1 crates more (use --limit N)";

#[test]
fn finds_the_exact_crate_version() {
    assert_eq!(
        parse_latest_version(SEARCH_OUTPUT, "cargo-cgp"),
        Some("0.2.0".to_owned())
    );
}

#[test]
fn does_not_match_a_lookalike_crate() {
    // Querying a name only present as a prefix of others returns nothing for it.
    assert_eq!(
        parse_latest_version("cargo-cgp-driver = \"0.3.0\"", "cargo-cgp"),
        None
    );
}

#[test]
fn absent_crate_yields_none() {
    assert_eq!(
        parse_latest_version("something-else = \"1.0.0\"", "cargo-cgp"),
        None
    );
}

#[test]
fn newer_version_is_newer() {
    assert!(is_newer("0.2.0", "0.1.0").unwrap());
    assert!(is_newer("1.0.0", "0.9.9").unwrap());
}

#[test]
fn equal_and_older_are_not_newer() {
    assert!(!is_newer("0.1.0", "0.1.0").unwrap());
    assert!(!is_newer("0.1.0", "0.2.0").unwrap());
}

#[test]
fn prerelease_orders_below_its_release() {
    // A prerelease is older than the final release of the same version.
    assert!(is_newer("0.1.0", "0.1.0-nightly.1").unwrap());
    assert!(!is_newer("0.1.0-nightly.1", "0.1.0").unwrap());
}

#[test]
fn invalid_version_errors() {
    assert!(is_newer("not-a-version", "0.1.0").is_err());
}
