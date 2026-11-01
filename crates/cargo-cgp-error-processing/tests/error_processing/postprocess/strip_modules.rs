//! Tests for stripping module qualifiers.
//!
//! Each transform reads plain text and gives `Some` when it changed it, so it is driven directly
//! over the case under test, with no diagnostic wrapper and no compiler.

use cargo_cgp_error_processing::{AmbiguousNames, strip_module_paths, strip_unambiguous_paths};

// Property: for all `a::b::C` runs, `strip_module_paths` keeps the run from its first uppercase
// segment.
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

// Property: an item inside a type or trait keeps its qualifier, and a run with no uppercase segment
// keeps its last segment.
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

// Property: `strip_unambiguous_paths` asks about the path to the item the run names, not the whole
// run.
#[test]
fn unambiguous_path_strip_reads_the_item_a_method_belongs_to() {
    // The ambiguity check reads the path to the type, `finance::Quantity`, not the whole run.
    let ambiguous = AmbiguousNames::new(
        [("Quantity".to_owned(), vec!["finance::Quantity".to_owned()])],
        [],
    );
    assert_eq!(
        strip_unambiguous_paths("`finance::Quantity::new`", &ambiguous).as_deref(),
        Some("`Quantity::new`"),
    );
}

// Property: only identifier runs change: a lone identifier, a primitive, and a `>::Assoc` tail
// stay.
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

// Property: for all text, a run inside a string literal stays.
#[test]
fn strip_module_paths_skips_string_literals() {
    // A `::` inside a quoted literal is not a module path.
    assert_eq!(strip_module_paths("Symbol!(\"a::b\")"), None);
}

// Property: a run right after a `use ` keeps every segment, and a word only ending in `use` does
// not count.
#[test]
fn strip_keeps_a_use_statement_path_whole() {
    // A `use` path is code the reader types, so its module segments stay, while the same path
    // elsewhere in the message, and a word merely ending in `use`, still collapse.
    let text = "call `downcast_fields` (from `use cgp::core::field::impls::CanDowncastFields;`) on \
                `cgp::core::field::impls::CanDowncastFields`, reuse a::B";
    assert_eq!(
        strip_unambiguous_paths(text, &AmbiguousNames::default()).as_deref(),
        Some(
            "call `downcast_fields` (from `use cgp::core::field::impls::CanDowncastFields;`) on \
             `CanDowncastFields`, reuse B"
        ),
    );
    assert_eq!(
        strip_module_paths("`use crate::components::GreeterComponent;`"),
        None
    );
}

// Property: for all text, characters outside a run are copied whole.
#[test]
fn strip_module_paths_preserves_multibyte_box_drawing() {
    // The dependency tree's `└─` characters are multi-byte UTF-8; stripping a module path on the
    // same line must copy them whole, never split them into invalid bytes.
    assert_eq!(
        strip_module_paths("  └─ the trait bound `f64: std::cmp::Eq` is not satisfied").as_deref(),
        Some("  └─ the trait bound `f64: Eq` is not satisfied"),
    );
}

// Property: a path naming the item a glob brings in collapses, a path naming another item keeps its
// qualifier, and a glob-imported name whose items could not be read keeps none.
#[test]
fn a_glob_imported_name_keeps_the_qualifier_of_another_item() {
    // The crate glob-imports `Error` naming `backend::Error`, so a bare `Error` reads as that item:
    // its own path collapses, and `std::io::Error` keeps its qualifier.
    let ambiguous = AmbiguousNames::new(
        [("Error".to_owned(), vec!["backend::Error".to_owned()])],
        [],
    );
    assert_eq!(
        strip_unambiguous_paths(
            "`CanRaiseError<std::io::Error>` and `backend::Error`",
            &ambiguous
        )
        .as_deref(),
        Some("`CanRaiseError<std::io::Error>` and `Error`"),
    );
    // A glob-imported name whose items could not be read keeps no qualifier.
    let unknown = AmbiguousNames::new([("Error".to_owned(), Vec::new())], []);
    assert_eq!(
        strip_unambiguous_paths("`std::io::Error`", &unknown).as_deref(),
        Some("`Error`"),
    );
}

// Property: a run collapses when rooted at `cgp`, when it ends in a CGP encoding name, or when
// `keeps_qualifier` says the bare name is safe; every other run stays.
#[test]
fn unambiguous_path_strip_keeps_a_disambiguating_qualifier() {
    // `cgp::prelude::HasErrorType` is CGP's and `Quantity` glob-imported, so both lose their
    // qualifier; `Error` and `Rectangle` are neither, so rustc qualified them for being ambiguous.
    let ambiguous = AmbiguousNames::new(
        [("Quantity".to_owned(), vec!["finance::Quantity".to_owned()])],
        [],
    );
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
