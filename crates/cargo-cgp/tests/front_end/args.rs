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

#[test]
fn unicode_arguments_pass_through() {
    use std::ffi::OsString;

    use cargo_cgp::args::utf8_args;

    let args = utf8_args([OsString::from("cargo-cgp"), OsString::from("check")]).unwrap();
    assert_eq!(args, ["cargo-cgp", "check"]);
}

#[cfg(unix)]
#[test]
fn a_non_unicode_argument_is_an_error_naming_it() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    use cargo_cgp::args::utf8_args;

    let error = utf8_args([
        OsString::from("cargo-cgp"),
        OsString::from("check"),
        OsString::from_vec(b"--manifest-path=\xff".to_vec()),
    ])
    .expect_err("a non-Unicode argument is rejected, not a panic");
    let message = error.to_string();
    assert!(message.contains("argument 2"), "{message}");
    assert!(message.contains("--manifest-path="), "{message}");
}
