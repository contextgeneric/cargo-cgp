//! Tests for the tool passes' environment ([`cargo_cgp_ui_tests::harness::tool_env`]).

use std::num::NonZeroUsize;

use cargo_cgp_ui_tests::harness::tool_env;
use cargo_cgp_ui_tests::options::Threads;

fn env(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn eight() -> Threads {
    Threads::Count(NonZeroUsize::new(8).unwrap())
}

// Property: with no fixture `RUSTFLAGS`, the ambient flags come first and the thread flag last.
#[test]
fn the_thread_flag_follows_the_ambient_flags() {
    assert_eq!(
        tool_env(&[], Some("-Dwarnings"), eight()),
        env(&[("RUSTFLAGS", "-Dwarnings -Zthreads=8")])
    );
    assert_eq!(
        tool_env(&[], None, Threads::ONE),
        env(&[("RUSTFLAGS", "-Zthreads=1")])
    );
}

// Property: a fixture's `//@env: RUSTFLAGS=…` replaces the ambient flags, as setting the variable
//   does, and keeps the thread flag after it; the fixture's other variables keep their order.
#[test]
fn a_fixtures_rustflags_replace_the_ambient_ones() {
    let fixture = env(&[
        ("RUSTFLAGS", "-Ztreat-err-as-bug=2"),
        ("RUST_BACKTRACE", "0"),
        ("RUSTC_ICE", "0"),
    ]);
    assert_eq!(
        tool_env(&fixture, Some("-Dwarnings"), eight()),
        env(&[
            ("RUST_BACKTRACE", "0"),
            ("RUSTC_ICE", "0"),
            ("RUSTFLAGS", "-Ztreat-err-as-bug=2 -Zthreads=8"),
        ])
    );
}

// Property: rustc's default adds no thread count, and with no other flags leaves `RUSTFLAGS`
//   unset rather than empty.
#[test]
fn the_default_mode_adds_no_thread_count() {
    assert_eq!(
        tool_env(&[], Some("-Dwarnings"), Threads::Default),
        env(&[("RUSTFLAGS", "-Dwarnings")])
    );
    assert_eq!(tool_env(&[], Some("  "), Threads::Default), env(&[]));
}
