//! Tests for process-argument normalization ([`cargo_cgp::args::strip_subcommand`]).

use cargo_cgp::args::strip_subcommand;

fn strip(args: &[&str]) -> Vec<String> {
    strip_subcommand(args.iter().map(|s| s.to_string()), "cgp")
}

#[test]
fn strips_inserted_subcommand() {
    assert_eq!(strip(&["cargo-cgp", "cgp", "check"]), ["check"]);
}

#[test]
fn handles_direct_invocation() {
    assert_eq!(strip(&["cargo-cgp", "check"]), ["check"]);
}

#[test]
fn keeps_a_later_matching_token() {
    assert_eq!(strip(&["cargo-cgp", "check", "cgp"]), ["check", "cgp"]);
}

#[test]
fn empty_when_only_program_name() {
    assert!(strip(&["cargo-cgp"]).is_empty());
}
