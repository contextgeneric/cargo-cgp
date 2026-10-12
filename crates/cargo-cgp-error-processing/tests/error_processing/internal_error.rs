//! Tests for the note on a diagnostic shown untransformed after an internal error.

use cargo_cgp_error_processing::internal_error_note;

#[test]
fn names_the_panic_and_where_to_report_it() {
    assert_eq!(
        internal_error_note(
            "called `Option::unwrap()` on a `None` value at src/emitter.rs:10:5",
            "https://example.org/issues",
        ),
        "cargo-cgp could not transform this diagnostic because of an internal error, so it is \
         shown as the compiler reported it: called `Option::unwrap()` on a `None` value at \
         src/emitter.rs:10:5\n\
         this is a bug in cargo-cgp; please report it at https://example.org/issues",
    );
}
