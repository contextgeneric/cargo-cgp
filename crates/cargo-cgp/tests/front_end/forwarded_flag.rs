//! Tests for reading a flag out of the forwarded arguments.

use cargo_cgp::expand::{forwards_profile, take_item};
use cargo_cgp::launch::{
    FlagScan, forwarded_config, forwarded_manifest_path, forwards_target_dir, scan_flag,
};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// Property: for all args, a spaced flag with no argument after it counts as present for the readers
// that ask whether it was given, and gives no value to the readers that ask for its value.
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

// Property: for all args, the argument after a spaced flag is that flag's value, even when it looks
// like the flag again.
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

// Property: for all args, `take_item` reports the first mistake in argument order.
#[test]
fn the_first_item_mistake_is_the_one_reported() {
    let needs_a_path = "`--item` needs the path of a module or item to expand";
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

// Property: for all args, a flag that only starts with the name of a flag is another flag.
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

// Concern: a separator could end the scan, which would hide an `--item` a caller's whole list holds.
// Handled: `scan_flag` treats `--` as an ordinary argument, and each caller chooses what it passes.
#[test]
fn a_scan_reads_past_a_separator() {
    assert_eq!(
        scan_flag(&args(&["--", "--f=x"]), "--f"),
        FlagScan {
            values: vec![Some("x".to_owned())],
            remainder: args(&["--"]),
        }
    );
}
