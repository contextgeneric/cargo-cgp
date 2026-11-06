//! Tests for assembling a wrapped cargo command's arguments
//! ([`cargo_cgp::launch::cargo_invocation`]).

use std::ffi::OsString;
use std::path::Path;

use cargo_cgp::launch::{cargo_invocation, split_forwarded};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn strings(args: Vec<OsString>) -> Vec<String> {
    args.into_iter()
        .map(|arg| arg.into_string().unwrap())
        .collect()
}

#[test]
fn a_check_forwards_the_arguments_then_the_target_directory() {
    let built = cargo_invocation(
        "check",
        &args(&["--workspace"]),
        &[],
        Some(Path::new("/proj/target/cgp")),
        &[],
    );
    assert_eq!(
        strings(built),
        ["check", "--workspace", "--target-dir", "/proj/target/cgp"]
    );
}

#[test]
fn an_expansion_puts_the_marker_flags_after_one_separator() {
    let built = cargo_invocation(
        "rustc",
        &args(&["--lib"]),
        &args(&["--profile", "check"]),
        Some(Path::new("/t/cgp")),
        &args(&["--cgp-expand=/o/expansion.rs"]),
    );
    assert_eq!(
        strings(built),
        [
            "rustc",
            "--lib",
            "--profile",
            "check",
            "--target-dir",
            "/t/cgp",
            "--",
            "--cgp-expand=/o/expansion.rs"
        ]
    );
}

#[test]
fn a_users_separator_is_kept_single_with_the_tools_flags_on_the_right_sides() {
    // `cargo cgp expand --lib -- -Zfoo`: the user's rustc flag and the marker flags share one `--`,
    // and the tool's cargo flags stay in front of it, where cargo reads them.
    let built = cargo_invocation(
        "rustc",
        &args(&["--lib", "--", "-Zfoo"]),
        &args(&["--profile", "check"]),
        Some(Path::new("/t/cgp")),
        &args(&["--cgp-expand=/o/expansion.rs"]),
    );
    assert_eq!(
        strings(built),
        [
            "rustc",
            "--lib",
            "--profile",
            "check",
            "--target-dir",
            "/t/cgp",
            "--",
            "-Zfoo",
            "--cgp-expand=/o/expansion.rs"
        ]
    );
}

#[test]
fn a_separator_with_nothing_after_it_is_kept() {
    let built = cargo_invocation("check", &args(&["--"]), &[], None, &[]);
    assert_eq!(strings(built), ["check", "--"]);
}

#[test]
fn the_forwarded_arguments_split_at_the_first_separator() {
    let given = args(&["-p", "app", "--", "-Zfoo", "--", "x"]);
    let (cargo, rustc) = split_forwarded(&given);
    assert_eq!(cargo, args(&["-p", "app"]));
    assert_eq!(rustc, Some(&args(&["-Zfoo", "--", "x"])[..]));
    assert_eq!(split_forwarded(&args(&["--lib"])).1, None);
}
