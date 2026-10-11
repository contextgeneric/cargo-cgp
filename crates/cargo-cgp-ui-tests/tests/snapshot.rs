//! Tests for snapshot review and the mismatch diff ([`cargo_cgp_ui_tests::snapshot`]).

use std::fs;

use cargo_cgp_ui_tests::snapshot::{Outcome, format_diff, review};

#[test]
fn a_missing_snapshot_is_a_mismatch() {
    let path =
        std::env::temp_dir().join(format!("cgp-ui-missing-{}.cgp.stderr", std::process::id()));
    let _ = fs::remove_file(&path);
    match review(&path, "", false, "cgp") {
        Outcome::Mismatch(text) => assert!(text.contains("missing snapshot"), "{text}"),
        _ => panic!("a missing snapshot must not pass as an empty one"),
    }
}

#[test]
fn blessing_writes_a_missing_snapshot() {
    let path = std::env::temp_dir().join(format!("cgp-ui-bless-{}.cgp.stderr", std::process::id()));
    let _ = fs::remove_file(&path);
    assert!(matches!(
        review(&path, "error: a", true, "cgp"),
        Outcome::Blessed
    ));
    assert_eq!(fs::read_to_string(&path).unwrap(), "error: a\n");
    assert!(matches!(
        review(&path, "error: a", false, "cgp"),
        Outcome::Ok
    ));
    let _ = fs::remove_file(&path);
}

#[test]
fn the_diff_shows_changed_lines_with_context() {
    let expected = "a\nb\nc\nd\ne\nf\ng";
    let actual = "a\nb\nc\nD\ne\nf\ng";
    let diff = format_diff("cgp", expected, actual);
    assert_eq!(
        diff,
        "  --- cgp: `-` committed snapshot, `+` actual output ---\n  ...\n    b\n    c\n  - d\n  + D\n    e\n    f\n  ...\n"
    );
}
