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

#[test]
fn collapses_chars_spines_regardless_of_truncation() {
    // The baseline renders a field-name spine non-deterministically: sometimes spelled to `Nil`,
    // sometimes truncated to `_` at a varying depth. Every form must collapse to one placeholder.
    let full = "`HasField<Symbol<4, Chars<'m', Chars<'a', Chars<'s', Chars<'s', Nil>>>>>>`";
    let truncated_deep = "`HasField<Symbol<4, Chars<'m', Chars<'a', Chars<'s', Chars<'s', _>>>>>>`";
    let truncated_shallow = "`HasField<Symbol<4, Chars<'m', Chars<'a', _>>>>`";
    let expected = "`HasField<Symbol<4, Chars<..>>>`";
    for input in [full, truncated_deep, truncated_shallow] {
        assert_eq!(normalize(input, &harness_dir(), &cgp_root()), expected);
    }
}

#[test]
fn collapses_qualified_and_multiple_spines() {
    // The baseline's primary "is not implemented" line prints the fully-qualified
    // `cgp::prelude::Chars` path; the prefix (no angle brackets) is kept, only the spine
    // collapses — and each spine on the line collapses.
    let input = "`HasField<Symbol<1, cgp::prelude::Chars<'a', cgp::prelude::Chars<'b', Nil>>>>` \
                 and `HasField<Symbol<1, cgp::prelude::Chars<'c', _>>>`";
    let expected = "`HasField<Symbol<1, cgp::prelude::Chars<..>>>` and `HasField<Symbol<1, cgp::prelude::Chars<..>>>`";
    assert_eq!(normalize(input, &harness_dir(), &cgp_root()), expected);
}
