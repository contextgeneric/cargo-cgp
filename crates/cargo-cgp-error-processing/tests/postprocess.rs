//! Tests for the fallback post-processing text transforms.
//!
//! Each transform is a pure `&str -> Option<String>` (`Some` when it changed the text), so
//! it is driven directly over the case under test — no diagnostic wrapper, no compiler.

use cargo_cgp_error_processing::{
    context_has_hasfield_impls, postprocess_message, resugar_lists, resugar_path, resugar_symbol,
    rewrite_missing_fields, strip_cgp_prefixes, strip_module_paths,
};

#[test]
fn resugars_a_product_of_bare_types() {
    assert_eq!(
        resugar_lists("Cons<u64, Cons<String, Nil>>").as_deref(),
        Some("Product![u64, String]"),
    );
}

#[test]
fn resugars_a_sum_of_bare_types() {
    assert_eq!(
        resugar_lists("Either<u64, Either<f64, Void>>").as_deref(),
        Some("Sum![u64, f64]"),
    );
}

#[test]
fn resugars_a_product_of_fields_to_a_struct() {
    assert_eq!(
        resugar_lists(
            "Cons<Field<Symbol!(\"width\"), f64>, Cons<Field<Symbol!(\"height\"), f64>, Nil>>"
        )
        .as_deref(),
        Some("Struct! { width: f64, height: f64 }"),
    );
}

#[test]
fn resugars_a_sum_of_fields_to_an_enum() {
    assert_eq!(
        resugar_lists(
            "Either<Field<Symbol!(\"Rect\"), u64>, Either<Field<Symbol!(\"Circle\"), f64>, Void>>"
        )
        .as_deref(),
        Some("Enum! { Rect(u64), Circle(f64) }"),
    );
}

#[test]
fn a_mixed_list_stays_a_plain_product() {
    // Not every element is a `Field`, so the list keeps its `Product!` form rather than `Struct!`.
    assert_eq!(
        resugar_lists("Cons<u64, Cons<Field<Symbol!(\"x\"), u8>, Nil>>").as_deref(),
        Some("Product![u64, Field<Symbol!(\"x\"), u8>]"),
    );
}

#[test]
fn resugars_a_nested_list() {
    assert_eq!(
        resugar_lists("Cons<Either<u8, Void>, Nil>").as_deref(),
        Some("Product![Sum![u8]]"),
    );
}

#[test]
fn resugars_a_spine_embedded_in_a_message() {
    assert_eq!(
        resugar_lists("required for `Cons<u64, Nil>` to implement `Foo`").as_deref(),
        Some("required for `Product![u64]` to implement `Foo`"),
    );
}

#[test]
fn declines_a_spine_that_does_not_terminate() {
    // The tail is not the exact `Nil`/`Void` terminator, so the structural match fails and the
    // text is left alone rather than mis-rewritten.
    assert_eq!(resugar_lists("Cons<u64, RestOfList>"), None);
    assert_eq!(resugar_lists("Either<u64, Either<f64, Nil>>"), None);
}

#[test]
fn does_not_match_cons_inside_path_cons() {
    // `PathCons` ends in `Cons`, but its `Cons<` is not a standalone cell, so a `PathCons` spine is
    // left for the path resugaring rather than being read as a `Product!`.
    assert_eq!(resugar_lists("PathCons<Foo, Nil>"), None);
}

#[test]
fn full_chain_resugars_a_raw_field_spine_to_a_struct() {
    // End to end: the raw `Symbol` spine is resugared first, then the field list folds to `Struct!`.
    assert_eq!(
        postprocess_message(
            "Cons<Field<Symbol<1, Chars<'x', Nil>>, u64>, Nil>",
            false,
            false,
        )
        .as_deref(),
        Some("Struct! { x: u64 }"),
    );
}

#[test]
fn resugars_a_path_bare_without_the_macro_wrapper() {
    // With `wrap = false` (the rewrite form), a path shows as a bare `@…` rather than the
    // `Path!(@…)` macro form the resugaring fallback uses.
    assert_eq!(
        resugar_path(
            "RedirectLookup<App, PathCons<Symbol!(\"app\"), PathCons<GreeterComponent, Nil>>>",
            false,
        )
        .as_deref(),
        Some("RedirectLookup<App, @app.GreeterComponent>"),
    );
}

#[test]
fn resugars_an_open_tailed_path_bare() {
    assert_eq!(
        resugar_path(
            "PathCons<Symbol!(\"foo\"), PathCons<Symbol!(\"bar\"), _>>",
            false
        )
        .as_deref(),
        Some("@foo.bar.*"),
    );
}

#[test]
fn strips_module_paths_to_the_final_segment() {
    assert_eq!(
        strip_module_paths(
            "the trait `interfaces::api::ApiHandler` is not implemented for `contexts::app::MockApp`"
        )
        .as_deref(),
        Some("the trait `ApiHandler` is not implemented for `MockApp`"),
    );
}

#[test]
fn strip_module_paths_keeps_generics_and_bare_names() {
    // A qualified type inside generics is collapsed to its tail; a bare name and a primitive are
    // left alone; a turbofish and an associated-type `>::Assoc` tail are not identifier runs.
    assert_eq!(
        strip_module_paths("std::option::Option<[u8]>").as_deref(),
        Some("Option<[u8]>"),
    );
    assert_eq!(strip_module_paths("Rectangle"), None);
    assert_eq!(strip_module_paths("f64"), None);
    assert_eq!(
        strip_module_paths("<Foo as a::b::Trait>::Value").as_deref(),
        Some("<Foo as Trait>::Value"),
    );
}

#[test]
fn strip_module_paths_skips_string_literals() {
    // A `::` inside a quoted literal is not a module path.
    assert_eq!(strip_module_paths("Symbol!(\"a::b\")"), None);
}

#[test]
fn strip_module_paths_preserves_multibyte_box_drawing() {
    // The dependency tree's `└─` characters are multi-byte UTF-8; stripping a module path on the
    // same line must copy them whole, never split them into invalid bytes.
    assert_eq!(
        strip_module_paths("  └─ the trait bound `f64: std::cmp::Eq` is not satisfied").as_deref(),
        Some("  └─ the trait bound `f64: Eq` is not satisfied"),
    );
}

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
            "RedirectLookup<App, PathCons<Symbol!(\"app\"), PathCons<GreeterComponent, Nil>>>",
            true
        )
        .as_deref(),
        Some("RedirectLookup<App, Path!(@app.GreeterComponent)>"),
    );
}

#[test]
fn resugars_a_single_segment_path() {
    assert_eq!(
        resugar_path("PathCons<MyFooComponent, Nil>", true).as_deref(),
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
             PathCons<finance::QuantityTypeProviderComponent, Nil>>>",
            true
        )
        .as_deref(),
        Some("Path!(@app.finance.QuantityTypeProviderComponent)"),
    );
}

#[test]
fn resugar_path_skips_a_qualified_segment_with_a_lowercase_tail() {
    // A qualified tail that is lowercase and non-primitive is not a type `Path!` would keep;
    // it would have been a `Symbol`, so the segment does not round-trip and the path declines.
    assert_eq!(resugar_path("PathCons<foo::bar, Nil>", true), None);
}

#[test]
fn resugar_path_skips_a_qualified_segment_with_generics() {
    // A `::`-path whose tail carries generics is not a plain identifier, so it declines.
    assert_eq!(resugar_path("PathCons<foo::Bar<T>, Nil>", true), None);
}

#[test]
fn resugars_a_primitive_segment_as_a_type() {
    // A primitive segment is kept as the named type by `Path!`, so it round-trips bare.
    assert_eq!(
        resugar_path("PathCons<u32, Nil>", true).as_deref(),
        Some("Path!(@u32)"),
    );
}

#[test]
fn resugars_an_open_dispatch_path_with_a_generic_value_segment() {
    // An `open` statement dispatches a component on a *value type* segment that may carry generics
    // (`Vec<u8>`). `Path!` keeps it verbatim as a type, so it round-trips bare rather than declining
    // and leaving the raw `PathCons` spine.
    assert_eq!(
        resugar_path(
            "PathCons<ItemEncoderComponent, PathCons<Vec<u8>, Nil>>",
            true
        )
        .as_deref(),
        Some("Path!(@ItemEncoderComponent.Vec<u8>)"),
    );
}

#[test]
fn resugars_an_open_dispatch_path_with_a_reference_value_segment() {
    // A borrowed value type (`&Coord`, after region erasure) is likewise kept verbatim.
    assert_eq!(
        resugar_path(
            "PathCons<ValueDeserializerComponent, PathCons<&Coord, Nil>>",
            false
        )
        .as_deref(),
        Some("@ValueDeserializerComponent.&Coord"),
    );
}

#[test]
fn resugars_an_open_tailed_path_as_a_wildcard() {
    // An open-ended path ends in a generic "rest of path" parameter, which rustc renders as
    // `_`. It resugars to a trailing `.*` wildcard segment — the form seen in the
    // conflicting-wiring E0119 blocks over a duplicated `@`-path key.
    assert_eq!(
        resugar_path(
            "PathCons<Symbol!(\"foo\"), PathCons<Symbol!(\"bar\"), _>>",
            true
        )
        .as_deref(),
        Some("Path!(@foo.bar.*)"),
    );
}

#[test]
fn resugars_a_single_segment_open_tailed_path() {
    assert_eq!(
        resugar_path("PathCons<Symbol!(\"foo\"), _>", true).as_deref(),
        Some("Path!(@foo.*)"),
    );
}

#[test]
fn resugar_path_skips_a_non_nil_terminated_spine() {
    // A tail that is neither `Nil`, another `PathCons`, nor the open `_` placeholder is not a
    // path spine — a concrete type like `App` is a genuine mismatch, not an open tail.
    assert_eq!(resugar_path("PathCons<GreeterComponent, App>", true), None);
}

#[test]
fn resugar_path_skips_an_uppercase_symbol_segment() {
    // `Path!` would never encode `Foo` (capitalized) as a `Symbol`, so a `Symbol!("Foo")`
    // head did not come from a path and must not be resugared to a lowercase-style segment.
    assert_eq!(resugar_path("PathCons<Symbol!(\"Foo\"), Nil>", true), None);
}

#[test]
fn resugar_path_leaves_plain_text_alone() {
    assert_eq!(
        resugar_path("the trait bound `Foo: Bar` is not satisfied", true),
        None
    );
}

#[test]
fn postprocess_message_resugars_an_expanded_path() {
    // End to end through the chain: an expanded `Symbol` spine inside a `PathCons` is
    // resugared to a symbol and then folded into a `Path!`.
    let text = "PathCons<Symbol<3, Chars<'a', Chars<'p', Chars<'p', Nil>>>>, PathCons<GreeterComponent, Nil>>";
    assert_eq!(
        postprocess_message(text, false, false).as_deref(),
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
        postprocess_message(text, false, false).as_deref(),
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
        postprocess_message(text, false, false).as_deref(),
        Some("`#[derive(HasField)]` is required to access field `width` on `Rectangle`"),
    );
}

#[test]
fn postprocess_message_leaves_plain_rust_alone() {
    assert_eq!(
        postprocess_message("the trait bound `Foo: Bar` is not satisfied", false, false),
        None,
    );
}
