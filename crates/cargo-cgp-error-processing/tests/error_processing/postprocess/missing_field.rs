//! Tests for the missing-field rewrite.
//!
//! Each transform reads plain text and gives `Some` when it changed it, so it is driven directly
//! over the case under test, with no diagnostic wrapper and no compiler.

use cargo_cgp_error_processing::{context_has_hasfield_impls, rewrite_missing_fields};

// Property: an unmet clause followed by the inline landmark rewrites to `missing field`, and the
// landmark goes.
#[test]
fn missing_field_with_inline_landmark_absorbs_it() {
    // A single missing field: the "but trait … is implemented for it" landmark follows
    // the clause inline, so it is absorbed into the rewrite.
    let text = "help: the trait `HasField<Symbol!(\"height\")>` is not implemented for `Rectangle`\n\
         \x20     but trait `HasField<Symbol!(\"width\")>` is implemented for it\n\
         \x20 --> src/main.rs:45:10";
    assert!(context_has_hasfield_impls(text));
    assert_eq!(
        rewrite_missing_fields(text, true).as_deref(),
        Some("help: missing field `height` on `Rectangle`\n  --> src/main.rs:45:10"),
    );
}

// Property: an unmet clause with a separate `implements trait` note rewrites, and the note stays.
#[test]
fn missing_field_with_separate_impls_note_is_recognized() {
    // Several other fields: the landmark is a separate `implements trait HasField` note,
    // not inline. It still classifies as a single missing field; the note is left as-is.
    let text = "help: the trait `HasField<Symbol!(\"height\")>` is not implemented for `Rectangle`\n\
         \x20 --> src/main.rs:59:1\n\
         help: `Rectangle` implements trait `HasField<Tag>`";
    assert!(context_has_hasfield_impls(text));
    assert_eq!(
        rewrite_missing_fields(text, true).as_deref(),
        Some(
            "help: missing field `height` on `Rectangle`\n  --> src/main.rs:59:1\n\
             help: `Rectangle` implements trait `HasField<Tag>`"
        ),
    );
}

// Property: `context_has_hasfield_impls` reads the note whether or not rustc qualifies the trait.
#[test]
fn separate_impls_note_is_recognized_with_its_path() {
    // rustc keeps a glob-imported trait's path in the note, so the landmark reads
    // `cgp::prelude::HasField`; it still shows the context implementing `HasField`.
    let text = "help: `Pair` implements trait `cgp::prelude::HasField<Tag>`";
    assert!(context_has_hasfield_impls(text));
}

// Property: `context_has_hasfield_impls` reads the inline landmark whether or not rustc qualifies
// the trait.
#[test]
fn inline_landmark_is_recognized_with_its_path() {
    let text =
        "but trait `cgp::prelude::HasField<Symbol<4, Chars<'n', Nil>>>` is implemented for it";
    assert!(context_has_hasfield_impls(text));
}

// Property: without a landmark, the clause keeps rustc's wording.
#[test]
fn no_landmark_keeps_the_clause() {
    // No landmark at all: the context implements `HasField` for no field, so the text cannot tell
    // a missing derive from a missing field, and the clause keeps rustc's wording.
    let text = "help: the trait `HasField<Symbol!(\"width\")>` is not implemented for `Rectangle`\n\
         \x20 --> src/main.rs:41:1";
    assert!(!context_has_hasfield_impls(text));
    assert_eq!(rewrite_missing_fields(text, false), None);
}

// Property: a numeric tag or a generated context keeps rustc's wording, even with a landmark.
#[test]
fn a_numeric_tag_or_a_generated_context_keeps_the_clause() {
    // `Symbol!("0")` names no field: a tuple struct keys its fields `Index<0>`.
    let tuple = "help: the trait `HasField<Symbol!(\"0\")>` is not implemented for `Pair`\n\
         help: `Pair` implements trait `HasField<Tag>`";
    assert_eq!(rewrite_missing_fields(tuple, true), None);
    // `__Context__` is a macro's generic context, whose fix is a bound, not a field.
    let generated = "help: the trait `HasField<Symbol!(\"name\")>` is not implemented for \
         `__Context__`\n\
         but trait `HasField<Symbol!(\"age\")>` is implemented for it";
    assert_eq!(rewrite_missing_fields(generated, true), None);
}

// Property: for all text with no unmet `HasField` clause, the rewrite gives `None`.
#[test]
fn missing_field_leaves_unrelated_diagnostics_alone() {
    let text = "error[E0277]: the trait bound `Foo: Bar` is not satisfied";
    assert!(!context_has_hasfield_impls(text));
    assert_eq!(rewrite_missing_fields(text, false), None);
}
