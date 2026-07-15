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
fn missing_subcommand_errors() {
    assert!(dispatch(&[]).is_err());
}
