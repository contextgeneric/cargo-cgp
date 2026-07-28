//! Tests for the `//@aux-build:` directive parser.
//!
//! The guard against a malformed directive is the point of most of these. A
//! directive the parser does not recognize used to be silently dropped, which cost
//! four cross-crate fixtures their auxiliary crate — they failed on an unresolved
//! import for months, reading as ordinary snapshot staleness rather than as a
//! disabled test.

use std::fs;
use std::path::PathBuf;

use cargo_cgp_ui_tests::aux;

fn fixture(name: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("cargo-cgp-aux-tests");
    fs::create_dir_all(&dir).expect("creating the temp fixture directory");
    let path = dir.join(name);
    fs::write(&path, body).expect("writing the temp fixture");
    path
}

#[test]
fn reads_one_declared_crate() {
    let path = fixture(
        "one.rs",
        "//! A fixture.\n//@aux-build: cgp-test-crate-a\n\nfn main() {}\n",
    );
    assert_eq!(aux::declared(&path), vec!["cgp-test-crate-a".to_owned()]);
}

#[test]
fn reads_several_declared_crates_in_order() {
    let path = fixture(
        "several.rs",
        "//@aux-build: cgp-test-crate-a\n//@aux-build: cgp-test-crate-b\nfn main() {}\n",
    );
    assert_eq!(
        aux::declared(&path),
        vec!["cgp-test-crate-a".to_owned(), "cgp-test-crate-b".to_owned()]
    );
}

#[test]
fn a_fixture_without_the_directive_declares_nothing() {
    let path = fixture("none.rs", "//! A fixture.\nfn main() {}\n");
    assert!(aux::declared(&path).is_empty());
}

#[test]
fn a_missing_fixture_declares_nothing() {
    let path = std::env::temp_dir().join("cargo-cgp-aux-tests/does-not-exist.rs");
    assert!(aux::declared(&path).is_empty());
}

#[test]
#[should_panic(expected = "malformed `//@aux-build:` directive")]
fn a_space_after_the_slashes_is_rejected() {
    // The exact regression: a formatter put a space after `//`, and the directive
    // was silently ignored.
    let path = fixture(
        "spaced.rs",
        "// @aux-build: cgp-test-crate-a\nfn main() {}\n",
    );
    let _ = aux::declared(&path);
}

#[test]
#[should_panic(expected = "malformed `//@aux-build:` directive")]
fn a_doc_comment_directive_is_rejected() {
    let path = fixture("doc.rs", "//! @aux-build: cgp-test-crate-a\nfn main() {}\n");
    let _ = aux::declared(&path);
}

#[test]
fn prose_mentioning_the_syntax_is_not_a_directive() {
    // A header explaining the mechanism must stay legal, so only a comment that
    // *starts* with the directive is flagged.
    let path = fixture(
        "prose.rs",
        "//! A fixture opts in with a `//@aux-build: name` line.\nfn main() {}\n",
    );
    assert!(aux::declared(&path).is_empty());
}
