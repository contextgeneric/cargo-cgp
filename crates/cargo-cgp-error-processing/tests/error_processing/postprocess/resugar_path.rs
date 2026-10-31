//! Tests for the `Path!` resugaring.
//!
//! Each transform is a pure `&str -> Option<String>` (`Some` when it changed the text), so
//! it is driven directly over the case under test, with no diagnostic wrapper and no compiler.

use cargo_cgp_error_processing::resugar_path;

// Property: for all well-formed paths, `wrap = false` gives the bare `@...` form.
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

// Property: for all paths ending in a bare `_`, the bare form leaves the tail out.
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

// Property: for all well-formed paths, `wrap = true` gives `Path!(@...)` and only the spine
// changes.
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

// Property: a one-segment path resugars like a longer one.
#[test]
fn resugars_a_single_segment_path() {
    assert_eq!(
        resugar_path("PathCons<MyFooComponent, Nil>", true).as_deref(),
        Some("Path!(@MyFooComponent)"),
    );
}

// Property: a qualified type segment that is a plain type path is rendered as written.
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

// Property: a qualified segment with a lowercase, non-primitive tail declines the whole path.
#[test]
fn resugar_path_skips_a_qualified_segment_with_a_lowercase_tail() {
    // A qualified tail that is lowercase and non-primitive is not a type `Path!` would keep;
    // it would have been a `Symbol`, so the segment does not round-trip and the path declines.
    assert_eq!(resugar_path("PathCons<foo::bar, Nil>", true), None);
}

// Property: a qualified segment whose tail carries generics declines the whole path.
#[test]
fn resugar_path_skips_a_qualified_segment_with_generics() {
    // A `::`-path whose tail carries generics is not a plain identifier, so it declines.
    assert_eq!(resugar_path("PathCons<foo::Bar<T>, Nil>", true), None);
}

// Property: for all primitive names, a bare segment of that name round-trips as a type.
#[test]
fn resugars_a_primitive_segment_as_a_type() {
    // Every primitive segment is kept as the named type by `Path!`, so each round-trips bare.
    for primitive in PRIMITIVE_NAMES {
        assert_eq!(
            resugar_path(&format!("PathCons<{primitive}, Nil>"), true),
            Some(format!("Path!(@{primitive})")),
            "{primitive}",
        );
    }
}

/// The names `Path!` keeps as types, as `cgp`'s path segment rule lists them.
const PRIMITIVE_NAMES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "char", "bool", "str",
];

// Property: a bare lowercase segment that is not a real primitive declines, since `Path!` would
// read it as a symbol.
#[test]
fn resugar_path_declines_a_type_named_like_a_primitive() {
    // `f`, `u2`, and `i7` look like primitives but are none, so `Path!` would read each as a
    // symbol: a type of that name has no `Path!` spelling, and the path declines.
    assert_eq!(resugar_path("PathCons<f, Nil>", true), None);
    assert_eq!(resugar_path("PathCons<u2, Nil>", true), None);
    assert_eq!(resugar_path("PathCons<i7, Nil>", true), None);
}

// Property: a compound value segment with generics is rendered verbatim.
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

// Property: a reference value segment is rendered verbatim.
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

// Property: for all paths ending in a bare `_`, the macro form declines.
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

// Property: for all tails other than `Nil`, `PathCons`, and a bare `_`, the path declines.
#[test]
fn resugar_path_skips_a_non_nil_terminated_spine() {
    // A tail that is neither `Nil`, another `PathCons`, nor the open `_` placeholder is not a
    // path spine — a concrete type like `App` is a genuine mismatch, not an open tail.
    assert_eq!(resugar_path("PathCons<GreeterComponent, App>", true), None);
}

// Property: a `Symbol!` head that is not a lowercase, non-primitive identifier declines, since
// `Path!` would not have made it a symbol.
#[test]
fn resugar_path_skips_an_uppercase_symbol_segment() {
    // `Path!` would never encode `Foo` (capitalized) as a `Symbol`, so a `Symbol!("Foo")`
    // head did not come from a path and must not be resugared to a lowercase-style segment.
    assert_eq!(resugar_path("PathCons<Symbol!(\"Foo\"), Nil>", true), None);
}

// Property: for all text with no `PathCons<`, `resugar_path` gives `None`.
#[test]
fn resugar_path_leaves_plain_text_alone() {
    assert_eq!(
        resugar_path("the trait bound `Foo: Bar` is not satisfied", true),
        None
    );
}
