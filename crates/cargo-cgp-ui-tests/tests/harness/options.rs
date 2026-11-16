//! Tests for harness option parsing ([`cargo_cgp_ui_tests::options::Options`]).

use std::num::NonZeroUsize;
use std::time::Duration;

use cargo_cgp_ui_tests::options::{DEFAULT_TIMEOUT, Options, Threads};

fn opts(args: &[&str]) -> Options {
    Options::parse(args.iter().map(|s| s.to_string()))
}

#[test]
fn parses_flags_and_filters() {
    let o = opts(&["--bless", "hidden", "--print", "--list", "greet"]);
    assert!(o.bless);
    assert!(o.print);
    assert!(o.list);
    assert_eq!(o.filters, ["hidden", "greet"]);
}

#[test]
fn ignores_unknown_flags() {
    let o = opts(&["--nocapture", "--exact"]);
    assert!(o.filters.is_empty());
    assert!(!o.bless);
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

fn count(n: usize) -> Threads {
    Threads::Count(NonZeroUsize::new(n).unwrap())
}

// Property: without `--threads`, `--iterations`, or `--timeout`, a run is the sequential suite:
//   one thread, one iteration, and the default deadline.
#[test]
fn the_defaults_are_one_thread_and_one_iteration() {
    let o = opts(&["hidden"]);
    assert_eq!(o.threads, Threads::ONE);
    assert!(!o.threads.is_parallel());
    assert_eq!(o.iterations.get(), 1);
    assert_eq!(o.timeout, DEFAULT_TIMEOUT);
}

// Property: `--threads` takes a count or `default`, in the spaced and the glued form, and its
//   value is never read as a filter.
#[test]
fn parses_threads_in_every_form() {
    assert_eq!(opts(&["--threads", "8"]).threads, count(8));
    assert_eq!(opts(&["--threads=8"]).threads, count(8));
    assert_eq!(opts(&["--threads", "default"]).threads, Threads::Default);
    assert_eq!(opts(&["--threads=default"]).threads, Threads::Default);
    let o = opts(&["--threads", "1", "hidden"]);
    assert_eq!(o.threads, Threads::ONE);
    assert_eq!(o.filters, ["hidden"]);
}

// Property: a thread count adds `-Zthreads=N`, `default` adds nothing, and every mode but one
//   thread is parallel.
#[test]
fn a_thread_mode_names_its_flag() {
    assert_eq!(count(8).flag().as_deref(), Some("-Zthreads=8"));
    assert_eq!(Threads::ONE.flag().as_deref(), Some("-Zthreads=1"));
    assert_eq!(Threads::Default.flag(), None);
    assert!(count(2).is_parallel());
    assert!(Threads::Default.is_parallel());
}

#[test]
fn parses_iterations_and_timeout() {
    let o = opts(&["--iterations", "3", "--timeout=90"]);
    assert_eq!(o.iterations.get(), 3);
    assert_eq!(o.timeout, Duration::from_secs(90));
}

// Concern: a mistyped mode that fell back to the default would run the sequential suite and
//   report it as the parallel one.
// Handled: `parse_positive` panics on a missing, zero, or unparsable value.
#[test]
#[should_panic(expected = "`--threads` takes a positive number")]
fn a_zero_thread_count_is_rejected() {
    let _ = opts(&["--threads", "0"]);
}

#[test]
#[should_panic(expected = "`--iterations` takes a positive number")]
fn a_missing_iteration_count_is_rejected() {
    let _ = opts(&["--iterations"]);
}

#[test]
#[should_panic(expected = "`--timeout` takes a positive number")]
fn an_unparsable_timeout_is_rejected() {
    let _ = opts(&["--timeout", "soon"]);
}
