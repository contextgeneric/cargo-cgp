//! Tests for running a command with a deadline ([`cargo_cgp_ui_tests::harness::capture`]).
//!
//! These run real processes through `sh`, so they hold only on a Unix host, where the suite runs.

use std::process::Command;
use std::time::{Duration, Instant};

use cargo_cgp_ui_tests::harness::capture;

fn sh(script: &str) -> Command {
    let mut command = Command::new("sh");
    command.args(["-c", script]);
    command
}

// Property: the postcondition of `capture` for a command that ends in time: both streams are
//   captured, the status is kept, and it did not time out.
#[test]
fn a_command_that_ends_keeps_its_output_and_status() {
    let captured = capture(
        &mut sh("echo out; echo err >&2; exit 3"),
        Duration::from_secs(60),
    );
    assert_eq!(captured.stdout, "out\n");
    assert_eq!(captured.stderr, "err\n");
    assert!(!captured.success);
    assert!(!captured.timed_out);
}

// Property: the postcondition of `capture` for a command still running at the deadline: it
//   timed out, did not succeed, and the call returns soon after the deadline.
#[test]
fn a_command_past_its_deadline_is_killed() {
    let start = Instant::now();
    let captured = capture(
        &mut sh("echo started; sleep 60"),
        Duration::from_millis(300),
    );
    assert!(captured.timed_out);
    assert!(!captured.success);
    assert_eq!(captured.stdout, "started\n");
    assert!(
        start.elapsed() < Duration::from_secs(30),
        "{:?}",
        start.elapsed()
    );
}

// Concern: cargo runs rustc as a child, and a child left running after the deadline would hold
//   the output pipes open, so reading them would wait for it.
// Handled: the command runs in its own process group, and the timeout kills the group.
#[test]
fn a_timeout_also_ends_the_commands_children() {
    let start = Instant::now();
    let captured = capture(&mut sh("sleep 60 & wait"), Duration::from_millis(300));
    assert!(captured.timed_out);
    assert!(
        start.elapsed() < Duration::from_secs(30),
        "{:?}",
        start.elapsed()
    );
}
