//! Tests for the driver's help query ([`cargo_cgp_driver::help`]).

use cargo_cgp_driver::help::{help_text, wants_help};

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn wants_help_detects_the_flags() {
    assert!(wants_help(&args(&["cargo-cgp-driver", "--help"])));
    assert!(wants_help(&args(&["cargo-cgp-driver", "-h"])));
    assert!(!wants_help(&args(&["cargo-cgp-driver", "--version"])));
    assert!(!wants_help(&args(&["cargo-cgp-driver"])));
}

#[test]
fn help_text_explains_the_wrapper_role_and_flags() {
    let help = help_text();
    for expected in [
        "cargo-cgp-driver",
        "RUSTC_WORKSPACE_WRAPPER",
        "--help",
        "--version",
        "cargo cgp check",
    ] {
        assert!(
            help.contains(expected),
            "help text should mention `{expected}`:\n{help}"
        );
    }
}
