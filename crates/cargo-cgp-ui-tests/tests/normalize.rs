//! Tests for output normalization ([`cargo_cgp_ui_tests::normalize::normalize`]).

use std::path::PathBuf;

use cargo_cgp_ui_tests::normalize::normalize;

fn harness_dir() -> PathBuf {
    PathBuf::from("/home/x/proj/target/ui-harness")
}

fn cgp_root() -> PathBuf {
    PathBuf::from("/home/x/cgp")
}

#[test]
fn replaces_paths_and_drops_noise_lines() {
    let raw = "\
error: boom
  --> /home/x/cgp/crates/core/cgp-field/src/traits/has_field.rs:50:1
   = note: the full name for the type has been written to '/home/x/proj/target/ui-harness/target/debug/deps/ui-abc.long-type-9.txt'
   = note: consider using `--verbose` to print the full type name to the console
   = help: keep this
error: could not compile `ui` (bin \"ui\") due to 1 previous error";
    let out = normalize(raw, &harness_dir(), &cgp_root());
    assert_eq!(
        out,
        "\
error: boom
  --> $CGP/crates/core/cgp-field/src/traits/has_field.rs:50:1
   = help: keep this"
    );
}
