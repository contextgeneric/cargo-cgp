//! Tests for the `expand` subcommand's pure helpers. Running the command itself spawns cargo and
//! the driver, so it is exercised end to end rather than here.

use cargo_cgp::expand::{forwards_profile, output_path, take_item};

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

#[test]
fn takes_the_item_filter_in_either_form() {
    let (forwarded, item) = take_item(&args(&["--lib", "--item", "shapes::Rectangle"])).unwrap();
    assert_eq!(forwarded, ["--lib"]);
    assert_eq!(item.as_deref(), Some("shapes::Rectangle"));

    let (forwarded, item) = take_item(&args(&["--item=shapes", "--lib"])).unwrap();
    assert_eq!(forwarded, ["--lib"]);
    assert_eq!(item.as_deref(), Some("shapes"));
}

#[test]
fn everything_else_is_forwarded_untouched() {
    let given = args(&["-p", "app", "--lib", "--features", "extra"]);
    let (forwarded, item) = take_item(&given).unwrap();

    assert_eq!(forwarded, given);
    assert!(item.is_none());
}

#[test]
fn a_bare_word_is_left_to_cargo() {
    // A positional cannot be told from a cargo flag's value — `--bin release` names a binary — so the
    // filter is only ever the explicit flag, and a bare word passes through.
    let (forwarded, item) = take_item(&args(&["--bin", "my_module"])).unwrap();

    assert_eq!(forwarded, ["--bin", "my_module"]);
    assert!(item.is_none());
}

#[test]
fn a_missing_or_repeated_item_filter_names_the_flag() {
    for bad in [
        vec!["--item"],
        vec!["--item="],
        vec!["--lib", "--item"],
        vec!["--item", "a", "--item", "b"],
    ] {
        let error = take_item(&args(&bad)).expect_err(&format!("{bad:?} should be rejected"));
        assert!(
            error.to_string().contains("--item"),
            "a flag-shaped mistake should name the flag: {error}"
        );
    }
}

#[test]
fn a_crate_rooted_item_path_is_accepted() {
    // The front-end only checks the shape; the driver's parser strips the prefix.
    for spelling in [
        "crate::contexts::app",
        "::contexts::app",
        "self::contexts::app",
    ] {
        let (_, item) = take_item(&args(&["--item", spelling])).expect("a valid path");
        assert_eq!(item.as_deref(), Some(spelling));
    }
}

#[test]
fn a_malformed_item_path_names_the_path() {
    // Rejected here rather than after a build, so a typo costs nothing.
    for bad in [
        "not a path",
        "shapes::",
        "shapes:Rectangle",
        "shapes::<T>",
        "::",
    ] {
        let error =
            take_item(&args(&["--item", bad])).expect_err(&format!("{bad:?} should be rejected"));
        assert!(
            error.to_string().contains(bad) && error.to_string().contains("item path"),
            "the message should quote the path and say what was expected: {error}"
        );
    }
}
