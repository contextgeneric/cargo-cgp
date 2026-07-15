//! Tests for `update`'s pure logic — the sparse-index path convention
//! ([`sparse_index_path`]), version extraction ([`parse_versions`]), and the
//! channel-preserving update selection ([`select_update`]).

use cargo_cgp::update::{parse_versions, select_update, sparse_index_path};

fn versions(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn sparse_paths_follow_the_registry_convention() {
    assert_eq!(sparse_index_path("a"), "1/a");
    assert_eq!(sparse_index_path("ab"), "2/ab");
    assert_eq!(sparse_index_path("abc"), "3/a/abc");
    assert_eq!(sparse_index_path("cargo-cgp"), "ca/rg/cargo-cgp");
    assert_eq!(sparse_index_path("serde_json"), "se/rd/serde_json");
    assert_eq!(sparse_index_path("Cargo-CGP"), "ca/rg/cargo-cgp"); // lowercased
}

#[test]
fn parses_versions_skipping_yanked_and_junk() {
    let body = "\
{\"name\":\"cargo-cgp\",\"vers\":\"0.1.0\",\"yanked\":false}
{\"name\":\"cargo-cgp\",\"vers\":\"0.1.1\",\"yanked\":true}
{\"name\":\"cargo-cgp\",\"vers\":\"0.2.0\",\"yanked\":false}
not json
";
    assert_eq!(parse_versions(body), versions(&["0.1.0", "0.2.0"]));
}

#[test]
fn stable_install_updates_to_the_highest_stable_ignoring_prereleases() {
    // The core requirement: v0.1.0 → v0.1.1, never v0.1.2-alpha.
    let vs = versions(&["0.1.0", "0.1.1", "0.1.2-alpha", "0.2.0-beta.1"]);
    assert_eq!(
        select_update(&vs, "0.1.0").unwrap(),
        Some("0.1.1".to_owned())
    );
}

#[test]
fn stable_install_with_only_prereleases_ahead_does_not_update() {
    // A higher pre-release exists but no newer stable — stay put.
    let vs = versions(&["0.1.0", "0.1.1-alpha", "0.2.0-alpha"]);
    assert_eq!(select_update(&vs, "0.1.0").unwrap(), None);
}

#[test]
fn prerelease_install_updates_to_the_highest_prerelease() {
    // v0.1.0-alpha → v0.1.1-alpha, not the stable 0.1.0 below it, nor a stable above.
    let vs = versions(&["0.1.0-alpha", "0.1.1-alpha", "0.1.0", "0.1.1"]);
    assert_eq!(
        select_update(&vs, "0.1.0-alpha").unwrap(),
        Some("0.1.1-alpha".to_owned())
    );
}

#[test]
fn no_newer_version_does_not_update() {
    let vs = versions(&["0.1.0", "0.0.9"]);
    assert_eq!(select_update(&vs, "0.1.0").unwrap(), None);
    // Refuses a downgrade even if only older versions exist.
    assert_eq!(select_update(&versions(&["0.0.9"]), "0.1.0").unwrap(), None);
}

#[test]
fn invalid_current_version_errors() {
    assert!(select_update(&versions(&["0.1.0"]), "not-a-version").is_err());
}
