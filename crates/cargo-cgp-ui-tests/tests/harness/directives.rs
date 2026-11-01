//! Tests for the `//@` directive parser ([`cargo_cgp_ui_tests::directives`]).

use std::fs;
use std::path::PathBuf;

use cargo_cgp_ui_tests::directives::{self, Directives, parse};

#[test]
fn reads_every_directive() {
    let text = "\
//! A fixture.
//@aux-build: cgp-test-crate-a
//@aux-build: cgp-test-crate-b
//@rewrites-falsely: the module strip drops the paths
//@json
//@env: RUST_BACKTRACE=0
//@env: RUSTFLAGS=-Ztreat-err-as-bug=1
//@codes: CGP-E001 CGP-E110
//@quality: acceptable
fn main() {}
";
    assert_eq!(
        parse(text, "demo.rs"),
        Directives {
            aux_build: vec!["cgp-test-crate-a".into(), "cgp-test-crate-b".into()],
            rewrites_falsely: Some("the module strip drops the paths".into()),
            json: true,
            env: vec![
                ("RUST_BACKTRACE".into(), "0".into()),
                ("RUSTFLAGS".into(), "-Ztreat-err-as-bug=1".into()),
            ],
            codes: Some(vec!["CGP-E001".into(), "CGP-E110".into()]),
            quality: Some("acceptable".into()),
        }
    );
}

#[test]
fn a_fixture_without_directives_declares_none() {
    assert_eq!(
        parse("//! A fixture.\nfn main() {}\n", "demo.rs"),
        Directives::default()
    );
}

#[test]
fn commented_out_wiring_is_not_a_directive() {
    // A comment that starts with an `@`-path, such as a disabled wiring entry, stays legal.
    let text = "fn main() {}\n        // @ItemEncoderComponent.Vec<u8>: EncodeBytes,\n";
    assert_eq!(parse(text, "demo.rs"), Directives::default());
}

#[test]
#[should_panic(expected = "unknown `//@` directive")]
fn an_unknown_directive_is_rejected() {
    let _ = parse("//@jsn\nfn main() {}\n", "demo.rs");
}

#[test]
#[should_panic(expected = "malformed `//@` directive")]
fn a_spaced_directive_is_rejected() {
    let _ = parse("// @json\nfn main() {}\n", "demo.rs");
}

#[test]
#[should_panic(expected = "needs a reason")]
fn a_rewrites_falsely_marker_needs_a_reason() {
    let _ = parse("//@rewrites-falsely:\nfn main() {}\n", "demo.rs");
}

#[test]
fn codes_none_is_an_empty_list() {
    let directives = parse("//@codes: none\nfn main() {}\n", "demo.rs");
    assert_eq!(directives.codes, Some(Vec::new()));
}

#[test]
#[should_panic(expected = "`//@codes:` takes `none` or `CGP-Exxx` codes")]
fn a_malformed_code_is_rejected() {
    let _ = parse("//@codes: E0277\nfn main() {}\n", "demo.rs");
}

#[test]
#[should_panic(expected = "`//@quality:` takes one of")]
fn an_unknown_quality_is_rejected() {
    let _ = parse("//@quality: fine\nfn main() {}\n", "demo.rs");
}

#[test]
#[should_panic(expected = "`//@env:` takes `KEY=VALUE`")]
fn an_env_directive_needs_an_assignment() {
    let _ = parse("//@env: RUST_BACKTRACE\nfn main() {}\n", "demo.rs");
}

#[test]
#[should_panic(expected = "a directive declared twice")]
fn a_second_codes_directive_is_rejected() {
    let _ = parse(
        "//@codes: CGP-E001\n//@codes: CGP-E002\nfn main() {}\n",
        "demo.rs",
    );
}

#[test]
#[should_panic(expected = "a directive declared twice")]
fn a_second_quality_directive_is_rejected() {
    let _ = parse(
        "//@quality: acceptable\n//@quality: usability\nfn main() {}\n",
        "demo.rs",
    );
}

#[test]
#[should_panic(expected = "needs a crate name")]
fn an_aux_build_directive_needs_a_crate() {
    let _ = parse("//@aux-build:\nfn main() {}\n", "demo.rs");
}

// `//@aux-build:` read from a file. The guard against a malformed directive is the point of most of
// these: a directive the parser dropped silently would cost a cross-crate fixture its auxiliary crate,
// and the fixture's unresolved import would read as ordinary snapshot staleness rather than as a
// disabled test.

fn fixture(name: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("cargo-cgp-aux-tests");
    fs::create_dir_all(&dir).expect("creating the temp fixture directory");
    let path = dir.join(name);
    fs::write(&path, body).expect("writing the temp fixture");
    path
}

// Property: for all fixtures, a `//@aux-build:` line anywhere in the header declares its crate.
#[test]
fn reads_one_declared_crate() {
    let path = fixture(
        "one.rs",
        "//! A fixture.\n//@aux-build: cgp-test-crate-a\n\nfn main() {}\n",
    );
    assert_eq!(
        directives::read(&path).aux_build,
        vec!["cgp-test-crate-a".to_owned()]
    );
}

// Property: for all fixtures, `aux_build` holds the declared crates in the order they are written.
#[test]
fn reads_several_declared_crates_in_order() {
    let path = fixture(
        "several.rs",
        "//@aux-build: cgp-test-crate-a\n//@aux-build: cgp-test-crate-b\nfn main() {}\n",
    );
    assert_eq!(
        directives::read(&path).aux_build,
        vec!["cgp-test-crate-a".to_owned(), "cgp-test-crate-b".to_owned()]
    );
}

// Property: a fixture with no `//@aux-build:` line declares no crate.
#[test]
fn a_fixture_without_the_directive_declares_nothing() {
    let path = fixture("none.rs", "//! A fixture.\nfn main() {}\n");
    assert!(directives::read(&path).aux_build.is_empty());
}

// Property: a fixture that cannot be read declares no crate.
#[test]
fn a_missing_fixture_declares_nothing() {
    let path = std::env::temp_dir().join("cargo-cgp-aux-tests/does-not-exist.rs");
    assert!(directives::read(&path).aux_build.is_empty());
}

// Property: a directive with a space after the slashes is rejected, never silently dropped.
#[test]
#[should_panic(expected = "malformed `//@` directive")]
fn a_space_after_the_slashes_is_rejected() {
    // The exact regression: a formatter put a space after `//`, and the directive
    // was silently ignored.
    let path = fixture(
        "spaced.rs",
        "// @aux-build: cgp-test-crate-a\nfn main() {}\n",
    );
    let _ = directives::read(&path).aux_build;
}

// Property: a directive in a doc comment is rejected, never silently dropped.
#[test]
#[should_panic(expected = "malformed `//@` directive")]
fn a_doc_comment_directive_is_rejected() {
    let path = fixture("doc.rs", "//! @aux-build: cgp-test-crate-a\nfn main() {}\n");
    let _ = directives::read(&path).aux_build;
}

// Property: a comment that only mentions the syntax further along is not a directive.
#[test]
fn prose_mentioning_the_syntax_is_not_a_directive() {
    // A header explaining the mechanism must stay legal, so only a comment that
    // *starts* with the directive is flagged.
    let path = fixture(
        "prose.rs",
        "//! A fixture opts in with a `//@aux-build: name` line.\nfn main() {}\n",
    );
    assert!(directives::read(&path).aux_build.is_empty());
}
