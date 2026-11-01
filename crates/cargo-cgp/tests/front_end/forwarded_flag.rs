//! Tests for reading a flag out of the forwarded arguments.

use cargo_cgp::expand::{asks_for_help, forwards_profile, take_item};
use cargo_cgp::launch::{
    FlagScan, forwarded_config, forwarded_manifest_path, forwards_target_dir, scan_flag,
};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// Property (assumed from what each reader's doc asks of the arguments): for all args, a spaced flag
// with no argument after it counts as present for the readers that ask whether it was given, and
// gives no value to the readers that ask for its value.
#[test]
fn a_spaced_flag_at_the_end_is_present_without_a_value() {
    assert!(forwards_target_dir(&args(&["--target-dir"])));
    assert!(forwards_profile(&args(&["--lib", "--profile"])));
    assert_eq!(forwarded_manifest_path(&args(&["--manifest-path"])), None);
    assert!(forwarded_config(&args(&["--config"])).is_empty());
}

// Property (assumed from `forwarded_manifest_path`'s callers): for all args naming
// `--manifest-path` more than once, the first one decides.
#[test]
fn the_first_manifest_path_decides() {
    assert_eq!(
        forwarded_manifest_path(&args(&["--manifest-path=a", "--manifest-path", "b"])),
        Some("a".to_owned())
    );
}

// Property (assumed from how cargo reads a spaced flag): for all args, the argument after a spaced
// flag is that flag's value, even when it looks like the flag again.
#[test]
fn a_spaced_flags_value_is_never_read_as_a_flag() {
    assert_eq!(
        forwarded_config(&args(&["--config", "--config=x", "--config=y"])),
        ["--config=x", "y"]
    );
    assert_eq!(
        take_item(&args(&["--item", "--item=x"]))
            .unwrap_err()
            .to_string(),
        "`--item=x` is not an item path: expected `::`-separated identifiers, as in `shapes::Rectangle`"
    );
}

// Property (assumed from reading arguments left to right): for all args, `take_item` reports the
// first mistake in argument order.
#[test]
fn the_first_item_mistake_is_the_one_reported() {
    let needs_a_path = "`--item` needs the path of a module or item to expand";
    assert!(
        take_item(&args(&["--item", "a", "--item", "b", "--item"]))
            .unwrap_err()
            .to_string()
            .starts_with("`--item` was given twice"),
        "the repeat comes before the flag with no path"
    );
    assert_eq!(
        take_item(&args(&["--item", "a", "--item"]))
            .unwrap_err()
            .to_string(),
        needs_a_path
    );
    assert_eq!(
        take_item(&args(&["--item=", "--item", "a"]))
            .unwrap_err()
            .to_string(),
        needs_a_path
    );
}

// Property (assumed from how cargo names flags): for all args, a flag that only starts with the
// name of a flag is another flag.
#[test]
fn a_longer_flag_is_not_the_flag() {
    assert!(!forwards_target_dir(&args(&["--target-dir-extra=x"])));
    assert!(!forwards_profile(&args(&["--profiled"])));
    assert!(forwarded_config(&args(&["--configure=x"])).is_empty());
    assert_eq!(
        take_item(&args(&["--items=x"])).unwrap().0,
        args(&["--items=x"])
    );
}

// Property: for all args and flag, `scan_flag` gives the occurrences' values in order and leaves
// the other arguments in order.
#[test]
fn a_scan_gives_the_values_in_order_and_the_rest_untouched() {
    assert_eq!(
        scan_flag(
            &args(&["-p", "a", "--f", "x", "--lib", "--f=y", "--f="]),
            "--f"
        ),
        FlagScan {
            values: vec![
                Some("x".to_owned()),
                Some("y".to_owned()),
                Some(String::new())
            ],
            remainder: args(&["-p", "a", "--lib"]),
        }
    );
}

// Property: for all args, `scan_flag` reads only the arguments before the first `--`, since cargo
// hands the rest to rustc, and keeps the separator and all that follows in the remainder, in order.
#[test]
fn a_scan_stops_at_a_separator_and_keeps_what_follows() {
    assert_eq!(
        scan_flag(
            &args(&["--f=x", "--lib", "--", "--f=y", "--", "--f", "z"]),
            "--f"
        ),
        FlagScan {
            values: vec![Some("x".to_owned())],
            remainder: args(&["--lib", "--", "--f=y", "--", "--f", "z"]),
        }
    );
}

// Property: for all args, `--item` after a `--` is rustc's: `take_item` takes no path from it and
// forwards it whole.
#[test]
fn an_item_flag_after_a_separator_reaches_rustc() {
    assert_eq!(
        take_item(&args(&["--lib", "--", "--item", "a"])).unwrap(),
        (args(&["--lib", "--", "--item", "a"]), None)
    );
}

// Property: a spaced `--item` directly before a `--` has no value, since the `--` ends what cargo
// reads.
#[test]
fn a_separator_is_not_an_items_value() {
    assert_eq!(
        take_item(&args(&["--item", "--", "a"]))
            .unwrap_err()
            .to_string(),
        "`--item` needs the path of a module or item to expand"
    );
}

// Property: for all args, a profile flag after a `--` chooses no profile, since it is rustc's.
#[test]
fn a_profile_flag_after_a_separator_chooses_no_profile() {
    assert!(!forwards_profile(&args(&["--lib", "--", "--release"])));
    assert!(!forwards_profile(&args(&["--lib", "--", "--profile=dev"])));
}

// Property: for all args, `expand` answers a help flag only when cargo would read it.
#[test]
fn only_a_help_flag_before_a_separator_asks_for_help() {
    assert!(asks_for_help(&args(&["--lib", "--help"])));
    assert!(asks_for_help(&args(&["-h", "--", "x"])));
    assert!(!asks_for_help(&args(&["--lib", "--", "--help"])));
    assert!(!asks_for_help(&args(&["--lib"])));
}

#[test]
fn detects_spaced_target_dir() {
    assert!(forwards_target_dir(&args(&[
        "--workspace",
        "--target-dir",
        "out"
    ])));
}

#[test]
fn detects_equals_target_dir() {
    assert!(forwards_target_dir(&args(&[
        "--target-dir=out",
        "--workspace"
    ])));
}

#[test]
fn absent_target_dir_is_not_detected() {
    assert!(!forwards_target_dir(&args(&[
        "--workspace",
        "--all-targets"
    ])));
}

#[test]
fn a_target_dir_after_a_separator_is_not_cargos() {
    // Under `expand`, arguments after a user's `--` go to rustc, so they choose no target directory.
    assert!(!forwards_target_dir(&args(&[
        "--lib",
        "--",
        "--target-dir",
        "elsewhere"
    ])));
}

#[test]
fn finds_a_forwarded_manifest_path_in_either_form() {
    assert_eq!(
        forwarded_manifest_path(&args(&["--manifest-path", "a/Cargo.toml", "--workspace"])),
        Some("a/Cargo.toml".to_owned())
    );
    assert_eq!(
        forwarded_manifest_path(&args(&["--workspace", "--manifest-path=b/Cargo.toml"])),
        Some("b/Cargo.toml".to_owned())
    );
    assert_eq!(forwarded_manifest_path(&args(&["--workspace"])), None);
}
