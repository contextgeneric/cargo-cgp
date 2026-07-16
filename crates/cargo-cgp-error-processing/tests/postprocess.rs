//! Tests for the fallback post-processing text transforms.
//!
//! Each transform is a pure `&str -> Option<String>` (`Some` when it changed the text), so
//! it is driven directly over the case under test — no diagnostic wrapper, no compiler.

use cargo_cgp_error_processing::{
    context_has_hasfield_impls, postprocess_message, resugar_path, resugar_symbol,
    rewrite_missing_fields, strip_cgp_prefixes,
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
fn resugars_a_symbol_and_type_path() {
    // The common shape: a namespace path of one lowercase symbol segment and one component
    // marker, embedded in a provider type. Runs after `Symbol!` resugaring, so the head is
    // already `Symbol!("app")`.
    assert_eq!(
        resugar_path(
            "RedirectLookup<App, PathCons<Symbol!(\"app\"), PathCons<GreeterComponent, Nil>>>"
        )
        .as_deref(),
        Some("RedirectLookup<App, Path!(@app.GreeterComponent)>"),
    );
}

#[test]
fn resugars_a_single_segment_path() {
    assert_eq!(
        resugar_path("PathCons<MyFooComponent, Nil>").as_deref(),
        Some("Path!(@MyFooComponent)"),
    );
}

#[test]
fn resugars_a_module_qualified_type_segment_by_its_tail() {
    // In a multi-module crate rustc prints a component defined in a sub-module qualified, e.g.
    // `finance::QuantityTypeProviderComponent`. `Path!` writes only the bare name, so the
    // segment resugars to its final component and the whole path stays readable rather than a
    // raw `PathCons<…>` spine.
    assert_eq!(
        resugar_path(
            "PathCons<Symbol!(\"app\"), PathCons<Symbol!(\"finance\"), \
             PathCons<finance::QuantityTypeProviderComponent, Nil>>>"
        )
        .as_deref(),
        Some("Path!(@app.finance.QuantityTypeProviderComponent)"),
    );
}

#[test]
fn resugar_path_skips_a_qualified_segment_with_a_lowercase_tail() {
    // A qualified tail that is lowercase and non-primitive is not a type `Path!` would keep;
    // it would have been a `Symbol`, so the segment does not round-trip and the path declines.
    assert_eq!(resugar_path("PathCons<foo::bar, Nil>"), None);
}

#[test]
fn resugar_path_skips_a_qualified_segment_with_generics() {
    // A `::`-path whose tail carries generics is not a plain identifier, so it declines.
    assert_eq!(resugar_path("PathCons<foo::Bar<T>, Nil>"), None);
}

#[test]
fn resugars_a_primitive_segment_as_a_type() {
    // A primitive segment is kept as the named type by `Path!`, so it round-trips bare.
    assert_eq!(
        resugar_path("PathCons<u32, Nil>").as_deref(),
        Some("Path!(@u32)"),
    );
}

#[test]
fn resugars_an_open_tailed_path_as_a_wildcard() {
    // An open-ended path ends in a generic "rest of path" parameter, which rustc renders as
    // `_`. It resugars to a trailing `.*` wildcard segment — the form seen in the
    // conflicting-wiring E0119 blocks over a duplicated `@`-path key.
    assert_eq!(
        resugar_path("PathCons<Symbol!(\"foo\"), PathCons<Symbol!(\"bar\"), _>>").as_deref(),
        Some("Path!(@foo.bar.*)"),
    );
}

#[test]
fn resugars_a_single_segment_open_tailed_path() {
    assert_eq!(
        resugar_path("PathCons<Symbol!(\"foo\"), _>").as_deref(),
        Some("Path!(@foo.*)"),
    );
}

#[test]
fn resugar_path_skips_a_non_nil_terminated_spine() {
    // A tail that is neither `Nil`, another `PathCons`, nor the open `_` placeholder is not a
    // path spine — a concrete type like `App` is a genuine mismatch, not an open tail.
    assert_eq!(resugar_path("PathCons<GreeterComponent, App>"), None);
}

#[test]
fn resugar_path_skips_an_uppercase_symbol_segment() {
    // `Path!` would never encode `Foo` (capitalized) as a `Symbol`, so a `Symbol!("Foo")`
    // head did not come from a path and must not be resugared to a lowercase-style segment.
    assert_eq!(resugar_path("PathCons<Symbol!(\"Foo\"), Nil>"), None);
}

#[test]
fn resugar_path_leaves_plain_text_alone() {
    assert_eq!(
        resugar_path("the trait bound `Foo: Bar` is not satisfied"),
        None
    );
}

#[test]
fn postprocess_message_resugars_an_expanded_path() {
    // End to end through the chain: an expanded `Symbol` spine inside a `PathCons` is
    // resugared to a symbol and then folded into a `Path!`.
    let text = "PathCons<Symbol<3, Chars<'a', Chars<'p', Chars<'p', Nil>>>>, PathCons<GreeterComponent, Nil>>";
    assert_eq!(
        postprocess_message(text, false).as_deref(),
        Some("Path!(@app.GreeterComponent)"),
    );
}

#[test]
fn postprocess_message_resugars_an_open_tailed_path() {
    // The conflicting-wiring E0119 shape end to end: two prefixed, expanded `Symbol` spines
    // inside a `PathCons` whose open `_` tail is the impl's generic "rest of path" parameter.
    // Stripped, resugared to symbols, then folded into a wildcard `Path!`.
    let text = "PathCons<cgp::prelude::Symbol<3, cgp::prelude::Chars<'f', cgp::prelude::Chars<'o', cgp::prelude::Chars<'o', Nil>>>>, PathCons<Symbol<3, Chars<'b', Chars<'a', Chars<'r', Nil>>>>, _>>";
    assert_eq!(
        postprocess_message(text, false).as_deref(),
        Some("Path!(@foo.bar.*)"),
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
