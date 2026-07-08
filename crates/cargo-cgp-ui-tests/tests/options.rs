//! Tests for harness option parsing ([`cargo_cgp_ui_tests::options::Options`]).

use cargo_cgp_ui_tests::options::Options;

fn opts(args: &[&str]) -> Options {
    Options::parse(args.iter().map(|s| s.to_string()))
}

#[test]
fn parses_flags_and_filters() {
    let o = opts(&["--bless", "hidden", "--print", "greet"]);
    assert!(o.bless);
    assert!(o.print);
    assert_eq!(o.filters, ["hidden", "greet"]);
}

#[test]
fn ignores_unknown_flags() {
    let o = opts(&["--nocapture", "--exact"]);
    assert!(o.filters.is_empty());
}

#[test]
fn empty_filters_match_everything() {
    assert!(opts(&[]).matches("hidden/unsatisfied_dependency.rs"));
}

#[test]
fn filter_matches_by_substring() {
    let o = opts(&["hidden"]);
    assert!(o.matches("hidden/unsatisfied_dependency.rs"));
    assert!(!o.matches("ok/greet.rs"));
}
