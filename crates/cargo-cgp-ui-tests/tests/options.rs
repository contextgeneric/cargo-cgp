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
fn parses_process_only() {
    let o = opts(&["--process-only", "wiring"]);
    assert!(o.process_only);
    assert!(!o.bless);
    assert_eq!(o.filters, ["wiring"]);
}

#[test]
fn ignores_unknown_flags() {
    let o = opts(&["--nocapture", "--exact"]);
    assert!(o.filters.is_empty());
    assert!(!o.process_only);
    assert_eq!(o.jobs, None);
}

#[test]
fn parses_jobs_in_every_form() {
    assert_eq!(opts(&["--jobs", "4"]).jobs, Some(4));
    assert_eq!(opts(&["-j", "4"]).jobs, Some(4));
    assert_eq!(opts(&["--jobs=4"]).jobs, Some(4));
    assert_eq!(opts(&["-j=4"]).jobs, Some(4));
    assert_eq!(opts(&["-j4"]).jobs, Some(4));
}

#[test]
fn jobs_value_is_consumed_not_taken_as_a_filter() {
    // A `--jobs` value that fails to parse is still swallowed, so it never leaks into the
    // filters and silently narrows the run.
    let o = opts(&["--jobs", "lots", "hidden"]);
    assert_eq!(o.jobs, None);
    assert_eq!(o.filters, ["hidden"]);
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
