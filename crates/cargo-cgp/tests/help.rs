//! Tests for the front-end help texts ([`cargo_cgp::help::help_text`] and
//! [`cargo_cgp::help::expand_help_text`]).

use cargo_cgp::help::{expand_help_text, help_text, is_help_flag};

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

#[test]
fn top_level_help_points_at_both_subcommand_helps() {
    let help = help_text();

    assert!(help.contains("expand"), "{help}");
    // The two subcommands' options come from different places — ours for `expand`, cargo's for
    // `check` — so the top level has to name both, or a reader looks in the wrong one.
    assert!(help.contains("cargo cgp expand --help"), "{help}");
    assert!(help.contains("cargo cgp check --help"), "{help}");
}

#[test]
fn expand_help_documents_the_item_filter_and_target_selection() {
    let help = expand_help_text();
    for expected in [
        // The one flag cargo does not know, and therefore the reason this text exists.
        "--item <PATH>",
        "crate::",
        // The three selection rules a reader has to choose between.
        "a module",
        "a type",
        "a trait",
        // The trap: a package with several targets needs one named.
        "--lib",
        "--bin NAME",
        // Where the forwarded options are documented.
        "cargo rustc --help",
        // And the limit that sends a reader to the other command.
        "cargo cgp check",
    ] {
        assert!(
            help.contains(expected),
            "expand help should mention `{expected}`:\n{help}"
        );
    }
}
