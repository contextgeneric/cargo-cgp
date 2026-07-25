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
