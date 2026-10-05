//! Tests for subcommand dispatch error paths ([`cargo_cgp::run::dispatch`]). The success
//! paths spawn cargo/rustup and are exercised end to end by the UI suite, so only the
//! side-effect-free error branches are unit-tested here.

use cargo_cgp::run::dispatch;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn unknown_subcommand_errors() {
    let err = dispatch(&args(&["frobnicate"])).unwrap_err().to_string();
    assert!(
        err.contains("frobnicate"),
        "message should name the bad subcommand: {err}"
    );
}

#[test]
fn no_subcommand_shows_help() {
    // A bare `cargo cgp` prints the help and succeeds rather than erroring.
    assert_eq!(dispatch(&[]).unwrap(), 0);
}

#[test]
fn help_flag_shows_help() {
    assert_eq!(dispatch(&args(&["--help"])).unwrap(), 0);
    assert_eq!(dispatch(&args(&["-h"])).unwrap(), 0);
}

#[test]
fn version_flag_prints_the_version() {
    assert_eq!(dispatch(&args(&["--version"])).unwrap(), 0);
    assert_eq!(dispatch(&args(&["-V"])).unwrap(), 0);
}

#[test]
fn setup_and_update_answer_help_without_running() {
    // Both install things, so `--help` must print help rather than start the install.
    for subcommand in ["setup", "update"] {
        assert_eq!(dispatch(&args(&[subcommand, "--help"])).unwrap(), 0);
        assert_eq!(dispatch(&args(&[subcommand, "-h"])).unwrap(), 0);
    }
}

#[test]
fn setup_and_update_reject_arguments() {
    for subcommand in ["setup", "update"] {
        let err = dispatch(&args(&[subcommand, "--force"]))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("takes no arguments") && err.contains("--force"),
            "message should name the stray argument: {err}"
        );
    }
}
