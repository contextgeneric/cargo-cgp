//! Tests for output normalization
//! ([`cargo_cgp_ui_tests::normalize::{normalize, normalize_json}`]).

use std::path::PathBuf;

use cargo_cgp_ui_tests::normalize::{normalize, normalize_json};

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

#[test]
fn json_replaces_paths_without_dropping_lines() {
    // The JSON normalizer only rewrites paths — it must never drop a line, since a
    // diagnostic's rendered text (here mentioning "could not compile") lives inside the
    // JSON and dropping it would corrupt the value.
    let raw = "\
{
  \"rendered\": \"error at /home/x/cgp/src/lib.rs and /home/x/proj/target/ui-harness/src/main.rs\",
  \"note\": \"could not compile is part of this string\"
}";
    let out = normalize_json(raw, &harness_dir(), &cgp_root());
    assert_eq!(
        out,
        "\
{
  \"rendered\": \"error at $CGP/src/lib.rs and $DIR/src/main.rs\",
  \"note\": \"could not compile is part of this string\"
}"
    );
}
