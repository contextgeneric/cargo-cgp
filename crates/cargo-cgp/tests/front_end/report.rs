//! Tests for the front-end's message forms ([`cargo_cgp::report`]).

use anyhow::Context;
use cargo_cgp::report::{
    INSTALL_RUSTUP_HELP, RUN_SETUP_HELP, error_report, quoted_output, status, warning,
};

#[test]
fn an_error_without_a_cause_is_one_error_line() {
    let error = anyhow::anyhow!("no such command: `frobnicate`");
    assert_eq!(error_report(&error), "error: no such command: `frobnicate`");
}

#[test]
fn each_cause_follows_under_a_caused_by_of_its_own() {
    // cargo prints a cause chain this way: one `Caused by:` per cause, each indented.
    let error = Err::<(), _>(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No such file or directory (os error 2)",
    ))
    .context("failed to run `rustup toolchain install`")
    .context("failed to install the toolchain")
    .unwrap_err();
    assert_eq!(
        error_report(&error),
        "error: failed to install the toolchain\n\n\
         Caused by:\n  failed to run `rustup toolchain install`\n\n\
         Caused by:\n  No such file or directory (os error 2)"
    );
}

#[test]
fn a_cause_of_several_lines_is_indented_line_by_line() {
    let error = Err::<(), _>(anyhow::anyhow!("first line\n\nthird line"))
        .context("the top")
        .unwrap_err();
    assert_eq!(
        error_report(&error),
        "error: the top\n\nCaused by:\n  first line\n\n  third line"
    );
}

#[test]
fn another_programs_output_is_quoted_as_an_indented_block() {
    assert_eq!(
        quoted_output("error: toolchain 'x' is not installed\n"),
        ":\n\n  error: toolchain 'x' is not installed"
    );
    assert_eq!(quoted_output("one\ntwo"), ":\n\n  one\n  two");
    assert_eq!(quoted_output(" \n"), "");
}

#[test]
fn a_warning_and_a_status_line_take_cargos_forms() {
    assert_eq!(
        warning("the wrapper does not run"),
        "warning: the wrapper does not run"
    );
    assert_eq!(
        status("Installing", "cargo-cgp-driver 0.1.0"),
        "  Installing cargo-cgp-driver 0.1.0"
    );
    assert_eq!(status("Finished", "setup"), "    Finished setup");
}

#[test]
fn the_shared_helps_are_help_lines() {
    assert_eq!(RUN_SETUP_HELP, "help: run `cargo cgp setup`");
    assert!(INSTALL_RUSTUP_HELP.starts_with("help: install rustup"));
}
