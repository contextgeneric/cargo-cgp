//! Tests for the `expand` subcommand's pure helpers. Running the command itself spawns cargo and
//! the driver, so it is exercised end to end rather than here.

use cargo_cgp::expand::{forwards_profile, output_path};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn detects_a_forwarded_release_flag() {
    assert!(forwards_profile(&args(&["--lib", "--release"])));
}

#[test]
fn detects_a_forwarded_profile_in_either_form() {
    assert!(forwards_profile(&args(&["--profile", "dev"])));
    assert!(forwards_profile(&args(&["--profile=dev"])));
}

#[test]
fn no_profile_flag_leaves_the_default_to_be_added() {
    assert!(!forwards_profile(&args(&["--lib", "--features", "extra"])));
    // A target named `release` is a value, not the flag, so it must not count.
    assert!(!forwards_profile(&args(&["--bin", "release"])));
}

#[test]
fn the_output_path_is_unique_per_process() {
    let path = output_path();
    let name = path.file_name().expect("a file name").to_string_lossy();

    assert!(
        name.contains(&std::process::id().to_string()),
        "two concurrent runs must not share an output file: {name}"
    );
    assert_eq!(
        path.extension().map(|e| e.to_string_lossy().into_owned()),
        Some("rs".to_owned())
    );
}
