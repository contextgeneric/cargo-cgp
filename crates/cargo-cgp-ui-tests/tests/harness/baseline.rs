//! Tests for the plain `cargo check` pass's `RUSTFLAGS`
//! ([`cargo_cgp_ui_tests::harness::baseline_rustflags`]).

use cargo_cgp_ui_tests::harness::{BASELINE_THREADS_FLAG, baseline_rustflags};

// Property: with no ambient `RUSTFLAGS`, or a blank one, the baseline's value is the one-thread
//   flag alone.
#[test]
fn uses_the_thread_flag_alone_without_ambient_flags() {
    assert_eq!(baseline_rustflags(None), BASELINE_THREADS_FLAG);
    assert_eq!(baseline_rustflags(Some("  ")), BASELINE_THREADS_FLAG);
}

// Property: the one-thread flag is appended after the ambient flags, so it follows, and overrides,
//   an ambient thread count.
#[test]
fn appends_the_thread_flag_after_ambient_flags() {
    assert_eq!(
        baseline_rustflags(Some("-Zthreads=8 -Dwarnings")),
        "-Zthreads=8 -Dwarnings -Zthreads=1"
    );
}
