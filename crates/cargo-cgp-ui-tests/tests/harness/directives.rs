//! Tests for the `//@` directive parser ([`cargo_cgp_ui_tests::directives`]).

use cargo_cgp_ui_tests::directives::{Directives, parse};

#[test]
fn reads_every_directive() {
    let text = "\
//! A fixture.
//@aux-build: cgp-test-crate-a
//@aux-build: cgp-test-crate-b
//@rewrites-falsely: the module strip drops the paths
//@json
//@env: CARGO_CGP_TEST_PANIC=flush
//@env: RUSTFLAGS=-Ztreat-err-as-bug=1
//@codes: CGP-E001 CGP-E106
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
                ("CARGO_CGP_TEST_PANIC".into(), "flush".into()),
                ("RUSTFLAGS".into(), "-Ztreat-err-as-bug=1".into()),
            ],
            codes: Some(vec!["CGP-E001".into(), "CGP-E106".into()]),
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
    let _ = parse("//@env: CARGO_CGP_TEST_PANIC\nfn main() {}\n", "demo.rs");
}
