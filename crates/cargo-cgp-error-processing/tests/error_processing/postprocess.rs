//! Tests for the fallback post-processing text transforms.
//!
//! Each transform is a pure `&str -> Option<String>` (`Some` when it changed the text), so
//! it is driven directly over the case under test — no diagnostic wrapper, no compiler.

use cargo_cgp_error_processing::{
    AmbiguousNames, Origin, context_has_hasfield_impls, postprocess_fragments, postprocess_message,
    resugar_lists, resugar_path, resugar_symbol, rewrite_missing_fields, strip_cgp_prefixes,
    strip_module_paths, strip_unambiguous_paths,
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
fn resugars_a_product_of_index_fields_to_a_tuple_struct() {
    assert_eq!(
        resugar_lists("Cons<Field<Index<0>, u64>, Cons<Field<Index<1>, String>, Nil>>").as_deref(),
        Some("Struct!(u64, String)"),
    );
}

#[test]
fn a_single_index_field_stays_a_plain_product() {
    // `Struct!(u64)` is the bare `u64`, so this one-element list has no `Struct!` spelling.
    assert_eq!(
        resugar_lists("Cons<Field<Index<0>, u64>, Nil>").as_deref(),
        Some("Product![Field<Index<0>, u64>]"),
    );
}

#[test]
fn a_field_name_that_is_not_an_identifier_stays_a_plain_product() {
    assert_eq!(
        resugar_lists("Cons<Field<Symbol!(\"foo bar\"), u8>, Nil>").as_deref(),
        Some("Product![Field<Symbol!(\"foo bar\"), u8>]"),
    );
}

#[test]
fn a_keyword_field_name_is_written_raw() {
    assert_eq!(
        resugar_lists("Cons<Field<Symbol!(\"type\"), u8>, Nil>").as_deref(),
        Some("Struct! { r#type: u8 }"),
    );
}

#[test]
fn each_variant_takes_its_shortest_spelling() {
    assert_eq!(
        resugar_lists(
            "Either<Field<Symbol!(\"Empty\"), Nil>, \
             Either<Field<Symbol!(\"Pair\"), Cons<Field<Index<0>, u8>, Cons<Field<Index<1>, u16>, Nil>>>, \
             Either<Field<Symbol!(\"Rect\"), Cons<Field<Symbol!(\"width\"), f64>, Nil>>, \
             Either<Field<Symbol!(\"One\"), Cons<Field<Index<0>, u8>, Nil>>, Void>>>>"
        )
        .as_deref(),
        Some("Enum! { Empty, Pair(u8, u16), Rect { width: f64 }, One(Product![Field<Index<0>, u8>]) }"),
    );
}

#[test]
fn a_sum_tagged_by_position_stays_a_plain_sum() {
    assert_eq!(
        resugar_lists("Either<Field<Index<0>, u8>, Void>").as_deref(),
        Some("Sum![Field<Index<0>, u8>]"),
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
            Origin::Untransformed,
            &AmbiguousNames::default()
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
fn resugars_an_open_tailed_path_bare_as_its_key() {
    // The bare form names a key, which the programmer wrote without the wildcard tail the macros
    // append, so the tail is left out — the form the conflicting-wiring E0119 blocks show.
    assert_eq!(
        resugar_path(
            "PathCons<Symbol!(\"foo\"), PathCons<Symbol!(\"bar\"), _>>",
            false
        )
        .as_deref(),
        Some("@foo.bar"),
    );
    assert_eq!(
        resugar_path("PathCons<Symbol!(\"foo\"), _>", false).as_deref(),
        Some("@foo"),
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
fn strip_module_paths_keeps_the_type_an_item_belongs_to() {
    // A module is named in lowercase and a type or trait in uppercase, so only the module segments
    // go: a trait's method keeps its trait, and `Self::Output` keeps `Self`.
    assert_eq!(
        strip_module_paths("required by a bound in `cgp::prelude::TryComputer::try_compute`")
            .as_deref(),
        Some("required by a bound in `TryComputer::try_compute`"),
    );
    assert_eq!(strip_module_paths("`Self::Output`"), None);
    // A run with no uppercase segment, such as a free function's path, keeps its last segment.
    assert_eq!(
        strip_module_paths("`std::mem::swap`").as_deref(),
        Some("`swap`")
    );
}

#[test]
fn unambiguous_path_strip_reads_the_item_a_method_belongs_to() {
    // The ambiguity check reads the path to the type, `finance::Quantity`, not the whole run.
    let ambiguous =
        AmbiguousNames::new([("Quantity".to_owned(), vec!["finance::Quantity".to_owned()])]);
    assert_eq!(
        strip_unambiguous_paths("`finance::Quantity::new`", &ambiguous).as_deref(),
        Some("`Quantity::new`"),
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
fn resugars_a_module_qualified_type_segment_as_written() {
    // rustc qualifies a type segment only when its bare name is ambiguous (`std::io::Error` in a
    // crate that also has an `Error`), and `Path!` parses the qualified type back, so the segment
    // keeps its qualifier rather than reading as another `Error`.
    assert_eq!(
        resugar_path(
            "PathCons<ErrorRaiserComponent, PathCons<std::io::Error, Nil>>",
            true
        )
        .as_deref(),
        Some("Path!(@ErrorRaiserComponent.std::io::Error)"),
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
fn declines_an_open_tailed_path_in_the_macro_form() {
    // An open-ended path ends in the macros' generic "rest of path" parameter, which rustc renders
    // as `_`. `Path!(@foo.bar)` would name the closed path, a different type, so the macro form
    // leaves the spine as it stands.
    assert_eq!(
        resugar_path(
            "PathCons<Symbol!(\"foo\"), PathCons<Symbol!(\"bar\"), _>>",
            true
        ),
        None,
    );
    assert_eq!(resugar_path("PathCons<Symbol!(\"foo\"), _>", true), None);
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
        postprocess_message(
            text,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("Path!(@app.GreeterComponent)"),
    );
}

#[test]
fn postprocess_message_resugars_an_open_tailed_path() {
    // The conflicting-wiring E0119 shape end to end: two prefixed, expanded `Symbol` spines
    // inside a `PathCons` whose open `_` tail is the impl's generic "rest of path" parameter.
    // A rewritten message folds it into the key its entry wrote; an untransformed one has no
    // `Path!` spelling for it, so it keeps the spine with its symbols resugared.
    let text = "PathCons<cgp::prelude::Symbol<3, cgp::prelude::Chars<'f', cgp::prelude::Chars<'o', cgp::prelude::Chars<'o', Nil>>>>, PathCons<Symbol<3, Chars<'b', Chars<'a', Chars<'r', Nil>>>>, _>>";
    assert_eq!(
        postprocess_message(text, false, Origin::Rewritten, &AmbiguousNames::default()).as_deref(),
        Some("@foo.bar"),
    );
    assert_eq!(
        postprocess_message(
            text,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("PathCons<Symbol!(\"foo\"), PathCons<Symbol!(\"bar\"), _>>"),
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
fn separate_impls_note_is_recognized_with_its_path() {
    // rustc keeps a glob-imported trait's path in the note, so the landmark reads
    // `cgp::prelude::HasField`; it still shows the context implementing `HasField`.
    let text = "help: `Pair` implements trait `cgp::prelude::HasField<Tag>`";
    assert!(context_has_hasfield_impls(text));
}

#[test]
fn inline_landmark_is_recognized_with_its_path() {
    let text =
        "but trait `cgp::prelude::HasField<Symbol<4, Chars<'n', Nil>>>` is implemented for it";
    assert!(context_has_hasfield_impls(text));
}

#[test]
fn no_landmark_keeps_the_clause() {
    // No landmark at all: the context implements `HasField` for no field, so the text cannot tell
    // a missing derive from a missing field, and the clause keeps rustc's wording.
    let text = "help: the trait `HasField<Symbol!(\"width\")>` is not implemented for `Rectangle`\n\
         \x20 --> src/main.rs:41:1";
    assert!(!context_has_hasfield_impls(text));
    assert_eq!(rewrite_missing_fields(text, false), None);
}

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

#[test]
fn missing_field_leaves_unrelated_diagnostics_alone() {
    let text = "error[E0277]: the trait bound `Foo: Bar` is not satisfied";
    assert!(!context_has_hasfield_impls(text));
    assert_eq!(rewrite_missing_fields(text, false), None);
}

#[test]
fn postprocess_message_chains_the_transforms() {
    // A prefixed, expanded `HasField<Symbol<…>>` clause with no similar-impl landmark: stripped
    // and resugared in one pass, and left in rustc's wording, since nothing shows the fix.
    let text = "the trait `HasField<cgp::prelude::Symbol<5, Chars<'w', Chars<'i', Chars<'d', Chars<'t', Chars<'h', Nil>>>>>>>` is not implemented for `Rectangle`";
    assert_eq!(
        postprocess_message(
            text,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("the trait `HasField<Symbol!(\"width\")>` is not implemented for `Rectangle`"),
    );
}

#[test]
fn postprocess_message_leaves_plain_rust_alone() {
    assert_eq!(
        postprocess_message(
            "the trait bound `Foo: Bar` is not satisfied",
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        ),
        None,
    );
}

// --- Fragmented messages -----------------------------------------------------------------
//
// rustc splits a message into styled fragments to highlight part of it, and splits at every
// difference when highlighting the difference between two types — shredding a CGP spine so no
// single fragment matches. `postprocess_fragments` reads them as the one line they render as.

#[test]
fn a_spine_split_across_fragments_is_resugared_as_a_whole() {
    // The shape rustc's "similar impl" hint produces, split at each differing character.
    let fragments = [
        "the trait `FromVariant<Symbol<3, Chars<'",
        "B",
        "', Chars<'",
        "a",
        "', Chars<'",
        "r",
        "', Nil>>>>>` is not implemented for `Big`",
    ];

    assert_eq!(
        postprocess_fragments(
            &fragments,
            false,
            Origin::Rewritten,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("the trait `FromVariant<Symbol!(\"Bar\")>` is not implemented for `Big`")
    );
}

#[test]
fn fragments_needing_no_joining_keep_their_styling() {
    // Each fragment is post-processed on its own by the caller before this runs, so a message
    // with nothing spanning a boundary must decline — flattening would drop rustc's
    // highlighting for no gain.
    let fragments = ["the trait `", "Foo", "` is not implemented for `Bar`"];

    assert_eq!(
        postprocess_fragments(
            &fragments,
            false,
            Origin::Rewritten,
            &AmbiguousNames::default()
        ),
        None
    );
}

#[test]
fn a_single_fragment_is_left_to_the_per_message_pass() {
    let fragments = ["Symbol<2, Chars<'x', Chars<'y', Nil>>>"];

    assert_eq!(
        postprocess_fragments(
            &fragments,
            false,
            Origin::Rewritten,
            &AmbiguousNames::default()
        ),
        None
    );
}

#[test]
fn an_untransformed_message_keeps_the_paths_rustc_printed() {
    // Two types sharing a name are told apart only by their paths, so a message rustc built and the
    // tool left alone keeps them; so does a glob-imported trait's path in a note.
    let same_name = "expected `a::Foo`, found `b::Foo`";
    assert_eq!(
        postprocess_message(
            same_name,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        ),
        None
    );
    let glob = "candidate #1 is defined in an impl of the trait `cgp::prelude::CanRaiseError`";
    assert_eq!(
        postprocess_message(
            glob,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        ),
        None
    );
}

#[test]
fn a_glob_imported_name_keeps_the_qualifier_of_another_item() {
    // The crate glob-imports `Error` naming `backend::Error`, so a bare `Error` reads as that item:
    // its own path collapses, and `std::io::Error` keeps its qualifier.
    let ambiguous = AmbiguousNames::new([("Error".to_owned(), vec!["backend::Error".to_owned()])]);
    assert_eq!(
        strip_unambiguous_paths(
            "`CanRaiseError<std::io::Error>` and `backend::Error`",
            &ambiguous
        )
        .as_deref(),
        Some("`CanRaiseError<std::io::Error>` and `Error`"),
    );
    // A glob-imported name whose items could not be read keeps no qualifier.
    let unknown = AmbiguousNames::new([("Error".to_owned(), Vec::new())]);
    assert_eq!(
        strip_unambiguous_paths("`std::io::Error`", &unknown).as_deref(),
        Some("`Error`"),
    );
}

#[test]
fn a_rewritten_message_keeps_only_qualifiers_that_disambiguate() {
    // A message the tool built drops every qualifier except on a name that names several items,
    // where the qualifier tells them apart.
    let ambiguous = AmbiguousNames::new([("App".to_owned(), vec!["app::App".to_owned()])]);
    assert_eq!(
        postprocess_message(
            "the consumer trait `cgp::prelude::CanRaiseError<std::io::Error>` for `app::App`",
            false,
            Origin::Rewritten,
            &ambiguous,
        )
        .as_deref(),
        Some("the consumer trait `CanRaiseError<std::io::Error>` for `App`"),
    );
}

#[test]
fn an_untransformed_message_with_a_cgp_encoding_is_cleaned_whole() {
    // Once a CGP encoding is recovered, the strip applies to the rest of that message too.
    let text = "`app::App: cgp::prelude::HasField<cgp::prelude::Symbol<1, Chars<'x', Nil>>>` is not satisfied";
    assert_eq!(
        postprocess_message(
            text,
            false,
            Origin::Untransformed,
            &AmbiguousNames::default()
        )
        .as_deref(),
        Some("`App: HasField<Symbol!(\"x\")>` is not satisfied"),
    );
}

#[test]
fn unambiguous_path_strip_keeps_a_disambiguating_qualifier() {
    // `cgp::prelude::HasErrorType` is CGP's and `Quantity` glob-imported, so both lose their
    // qualifier; `Error` and `Rectangle` are neither, so rustc qualified them for being ambiguous.
    let ambiguous =
        AmbiguousNames::new([("Quantity".to_owned(), vec!["finance::Quantity".to_owned()])]);
    assert_eq!(
        strip_unambiguous_paths(
            "`backend::Error` beside `cgp::prelude::HasErrorType` and `finance::Quantity`",
            &ambiguous,
        )
        .as_deref(),
        Some("`backend::Error` beside `HasErrorType` and `Quantity`"),
    );
    assert_eq!(
        strip_unambiguous_paths("`shape_a::Rectangle`", &ambiguous),
        None
    );
    // A CGP encoding type collapses through any crate's re-export, so the later stages fold it.
    assert_eq!(
        strip_unambiguous_paths("`app::prelude::Chars<'x', app::prelude::Nil>`", &ambiguous)
            .as_deref(),
        Some("`Chars<'x', Nil>`"),
    );
    assert_eq!(
        strip_unambiguous_paths("Symbol!(\"a::B\")", &AmbiguousNames::default()),
        None
    );
}
