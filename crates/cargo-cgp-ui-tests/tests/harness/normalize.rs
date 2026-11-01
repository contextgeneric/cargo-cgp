//! Tests for output normalization ([`cargo_cgp_ui_tests::normalize::normalize`]).

use std::path::PathBuf;

use cargo_cgp_ui_tests::normalize::{normalize, normalize_baseline};

fn harness_dir() -> PathBuf {
    PathBuf::from("/home/x/proj/target/ui-harness")
}

fn cgp_root() -> PathBuf {
    PathBuf::from("/home/x/cgp")
}

fn sysroot() -> PathBuf {
    PathBuf::from("/home/x/.rustup/toolchains/nightly-2026-09-14-aarch64-unknown-linux-gnu")
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
    let out = normalize(raw, &harness_dir(), &cgp_root(), &sysroot());
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
        assert_eq!(
            normalize_baseline(input, &harness_dir(), &cgp_root(), &sysroot()),
            expected
        );
    }
}

#[test]
fn collapses_qualified_and_multiple_spines() {
    // The baseline's primary "is not implemented" line prints the fully-qualified
    // `cgp::prelude::Chars` path; the prefix (no angle brackets) is kept, only the spine
    // collapses, and each spine on the line collapses.
    let input = "`HasField<Symbol<1, cgp::prelude::Chars<'a', cgp::prelude::Chars<'b', Nil>>>>` \
                 and `HasField<Symbol<1, cgp::prelude::Chars<'c', _>>>`";
    let expected = "`HasField<Symbol<1, cgp::prelude::Chars<..>>>` and `HasField<Symbol<1, cgp::prelude::Chars<..>>>`";
    assert_eq!(
        normalize_baseline(input, &harness_dir(), &cgp_root(), &sysroot()),
        expected
    );
}

#[test]
fn replaces_the_sysroot_in_a_standard_library_note() {
    // A diagnostic that points into the standard library names a path holding the user's home
    // directory, the pinned nightly, and the host target triple, none of which may reach a
    // snapshot. Whether rustc can show the source line depends on the `rust-src` component, which
    // `rust-toolchain.toml` pins for exactly that reason; the path itself is normalized here.
    let raw = "\
note: required by an implicit `Sized` bound in `Option`
   --> /home/x/.rustup/toolchains/nightly-2026-09-14-aarch64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:598:17";
    assert_eq!(
        normalize(raw, &harness_dir(), &cgp_root(), &sysroot()),
        "\
note: required by an implicit `Sized` bound in `Option`
   --> $SYSROOT/lib/rustlib/src/rust/library/core/src/option.rs:598:17"
    );
}

#[test]
fn the_tool_output_keeps_a_chars_spine() {
    // A raw spine in the tool's output means its resugaring declined, so the tool pass must not
    // collapse it the way the baseline's run-to-run truncation is collapsed.
    let spine = "`HasField<Symbol<1, Chars<'a', Nil>>>`";
    assert_eq!(
        normalize(spine, &harness_dir(), &cgp_root(), &sysroot()),
        spine
    );
}

#[test]
fn drops_the_coverage_failure_line() {
    // Under coverage instrumentation cargo adds its own failure line for the compiler process.
    let raw = "\
error: boom
error: process didn't exit successfully: `/home/x/rustc --crate-name ui` (exit status: 1)";
    assert_eq!(
        normalize(raw, &harness_dir(), &cgp_root(), &sysroot()),
        "error: boom"
    );
}

#[test]
fn replaces_the_thread_id_of_a_panic() {
    let raw = "\
thread 'rustc' (3683731) panicked at /rustc-dev/abc/compiler/rustc_errors/src/lib.rs:1506:17:
aborting after 2 errors
thread 'main' (12) panicked at src/main.rs:1:1:
thread 'rustc' panicked at src/main.rs:1:1:
note: thread 'rustc' (3683731) panicked";
    let out = normalize(raw, &harness_dir(), &cgp_root(), &sysroot());
    assert_eq!(
        out,
        "\
thread 'rustc' ($TID) panicked at /rustc-dev/abc/compiler/rustc_errors/src/lib.rs:1506:17:
aborting after 2 errors
thread 'main' ($TID) panicked at src/main.rs:1:1:
thread 'rustc' panicked at src/main.rs:1:1:
note: thread 'rustc' (3683731) panicked"
    );
}
