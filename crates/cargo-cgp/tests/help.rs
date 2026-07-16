//! Tests for the front-end help text ([`cargo_cgp::help::help_text`]).

use cargo_cgp::help::{help_text, is_help_flag};

#[test]
fn help_lists_the_subcommands_and_options() {
    let help = help_text();
    for expected in [
        "cargo-cgp",
        "Usage:",
        "check",
        "setup",
        "update",
        "-h, --help",
    ] {
        assert!(
            help.contains(expected),
            "help text should mention `{expected}`:\n{help}"
        );
    }
}

#[test]
fn recognizes_help_flags() {
    assert!(is_help_flag("--help"));
    assert!(is_help_flag("-h"));
    assert!(!is_help_flag("check"));
    assert!(!is_help_flag("--version"));
}
