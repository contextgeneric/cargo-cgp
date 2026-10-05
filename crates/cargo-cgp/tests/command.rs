//! Tests for the wrapped-launch helpers that are pure enough to pin directly.

use cargo_cgp::launch::forwards_target_dir;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn detects_spaced_target_dir() {
    assert!(forwards_target_dir(&args(&[
        "--workspace",
        "--target-dir",
        "out"
    ])));
}

#[test]
fn detects_equals_target_dir() {
    assert!(forwards_target_dir(&args(&[
        "--target-dir=out",
        "--workspace"
    ])));
}

#[test]
fn absent_target_dir_is_not_detected() {
    assert!(!forwards_target_dir(&args(&[
        "--workspace",
        "--all-targets"
    ])));
}

use std::path::PathBuf;

use cargo_cgp::launch::{forwarded_manifest_path, parse_target_directory};

#[test]
fn finds_a_forwarded_manifest_path_in_either_form() {
    assert_eq!(
        forwarded_manifest_path(&args(&["--manifest-path", "a/Cargo.toml", "--workspace"])),
        Some("a/Cargo.toml".to_owned())
    );
    assert_eq!(
        forwarded_manifest_path(&args(&["--workspace", "--manifest-path=b/Cargo.toml"])),
        Some("b/Cargo.toml".to_owned())
    );
    assert_eq!(forwarded_manifest_path(&args(&["--workspace"])), None);
}

#[test]
fn reads_the_target_directory_from_cargo_metadata() {
    let json = r#"{"packages":[],"target_directory":"/work/proj/target","version":1}"#;
    assert_eq!(
        parse_target_directory(json),
        Some(PathBuf::from("/work/proj/target"))
    );
    assert_eq!(parse_target_directory("not json"), None);
    assert_eq!(parse_target_directory(r#"{"packages":[]}"#), None);
}
