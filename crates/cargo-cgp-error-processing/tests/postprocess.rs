//! Tests for the fallback post-processing text transforms.
//!
//! Each transform is a pure `&str -> Option<String>` (`Some` when it changed the text), so
//! it is driven directly over the case under test — no diagnostic wrapper, no compiler.

use cargo_cgp_error_processing::{
    context_has_hasfield_impls, postprocess_message, resugar_symbol, rewrite_missing_fields,
    strip_cgp_prefixes,
};

#[test]
fn strips_known_cgp_prefixes() {
    assert_eq!(
        strip_cgp_prefixes(
            "HasField<cgp::prelude::Symbol<...>>, cgp::cgp_core::Foo, cgp::cgp_extra::Bar"
        )
        .as_deref(),
        Some("HasField<Symbol<...>>, Foo, Bar"),
    );
}

#[test]
fn strip_leaves_non_cgp_text_untouched() {
    assert_eq!(
        strip_cgp_prefixes("the trait bound `Rectangle: Greeter` is not satisfied"),
        None,
    );
}

#[test]
fn resugars_an_exact_symbol() {
    // `height` — six ASCII characters, so the length is 6. The surrounding `HasField<…>`
    // must survive with its own closing `>`.
    assert_eq!(
        resugar_symbol(
            "HasField<Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<'h', Chars<'t', Nil>>>>>>>>"
        )
        .as_deref(),
        Some("HasField<Symbol!(\"height\")>"),
    );
}

#[test]
fn resugars_the_empty_symbol() {
    assert_eq!(
        resugar_symbol("Symbol<0, Nil>").as_deref(),
        Some("Symbol!(\"\")")
    );
}

#[test]
fn resugar_skips_a_wrong_length() {
    // Declared length 3 but only two characters — not an exact match, so left alone.
    assert_eq!(
        resugar_symbol("Symbol<3, Chars<'x', Chars<'y', Nil>>>"),
        None
    );
}

#[test]
fn resugar_skips_a_foreign_symbol() {
    // A type named `Symbol` with a non-`Chars` payload must not be rewritten.
    assert_eq!(resugar_symbol("Symbol<2, SomeOtherType>"), None);
}

#[test]
fn resugar_leaves_other_symbol_uses_alone() {
    assert_eq!(
        resugar_symbol("the trait `Symbolic` is not implemented"),
        None
    );
}

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

#[test]
fn missing_derive_when_no_impls_present() {
    // No landmark at all: the context implements HasField for nothing, so the whole
    // derive is missing and the message points at the derive, not a single field.
    let text = "help: the trait `HasField<Symbol!(\"width\")>` is not implemented for `Rectangle`\n\
         \x20 --> src/main.rs:41:1";
    assert!(!context_has_hasfield_impls(text));
    assert_eq!(
        rewrite_missing_fields(text, false).as_deref(),
        Some(
            "help: `#[derive(HasField)]` is required to access field `width` on `Rectangle`\n\
             \x20 --> src/main.rs:41:1"
        ),
    );
}

#[test]
fn missing_field_leaves_unrelated_diagnostics_alone() {
    let text = "error[E0277]: the trait bound `Foo: Bar` is not satisfied";
    assert!(!context_has_hasfield_impls(text));
    assert_eq!(rewrite_missing_fields(text, false), None);
}

#[test]
fn postprocess_message_chains_the_transforms() {
    // A prefixed, expanded `HasField<Symbol<…>>` clause with no similar-impl landmark:
    // stripped, resugared, then rewritten as a missing derive in one pass.
    let text = "the trait `HasField<cgp::prelude::Symbol<5, Chars<'w', Chars<'i', Chars<'d', Chars<'t', Chars<'h', Nil>>>>>>>` is not implemented for `Rectangle`";
    assert_eq!(
        postprocess_message(text, false).as_deref(),
        Some("`#[derive(HasField)]` is required to access field `width` on `Rectangle`"),
    );
}

#[test]
fn postprocess_message_leaves_plain_rust_alone() {
    assert_eq!(
        postprocess_message("the trait bound `Foo: Bar` is not satisfied", false),
        None,
    );
}
