//! Tests for the `Product!`/`Sum!` and `Struct!`/`Enum!` resugaring.
//!
//! Each transform is a pure `&str -> Option<String>` (`Some` when it changed the text), so
//! it is driven directly over the case under test, with no diagnostic wrapper and no compiler.

use cargo_cgp_error_processing::resugar_lists;

// Property: for all spines `Cons<A, ..., Nil>` of non-`Field` elements, `resugar_lists` gives `Product![A, ...]`.
#[test]
fn resugars_a_product_of_bare_types() {
    assert_eq!(
        resugar_lists("Cons<u64, Cons<String, Nil>>").as_deref(),
        Some("Product![u64, String]"),
    );
}

// Property: for all spines `Either<A, ..., Void>` of non-`Field` elements, `resugar_lists` gives `Sum![A, ...]`.
#[test]
fn resugars_a_sum_of_bare_types() {
    assert_eq!(
        resugar_lists("Either<u64, Either<f64, Void>>").as_deref(),
        Some("Sum![u64, f64]"),
    );
}

// Property: for all products whose elements are all `Field`s with writable names, `resugar_lists` gives
// `Struct! { name: Type, ... }`.
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

// Property: for all sums whose elements are all `Field`s with writable names, `resugar_lists` gives
// `Enum! { Name(Type), ... }`.
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

// Property: for all products of two or more `Index` fields in order, `resugar_lists` gives `Struct!(A, B, ...)`.
#[test]
fn resugars_a_product_of_index_fields_to_a_tuple_struct() {
    assert_eq!(
        resugar_lists("Cons<Field<Index<0>, u64>, Cons<Field<Index<1>, String>, Nil>>").as_deref(),
        Some("Struct!(u64, String)"),
    );
}

// Property: a product whose shape has no exact `Struct!` spelling keeps the plain list form.
#[test]
fn a_single_index_field_stays_a_plain_product() {
    // `Struct!(u64)` is the bare `u64`, so this one-element list has no `Struct!` spelling.
    assert_eq!(
        resugar_lists("Cons<Field<Index<0>, u64>, Nil>").as_deref(),
        Some("Product![Field<Index<0>, u64>]"),
    );
}

// Property: a product with a field name no body can write keeps the plain list form.
#[test]
fn a_field_name_that_is_not_an_identifier_stays_a_plain_product() {
    assert_eq!(
        resugar_lists("Cons<Field<Symbol!(\"foo bar\"), u8>, Nil>").as_deref(),
        Some("Product![Field<Symbol!(\"foo bar\"), u8>]"),
    );
}

// Property: a field name that is a keyword appears in `Struct!` in its raw spelling.
#[test]
fn a_keyword_field_name_is_written_raw() {
    assert_eq!(
        resugar_lists("Cons<Field<Symbol!(\"type\"), u8>, Nil>").as_deref(),
        Some("Struct! { r#type: u8 }"),
    );
}

// Property: for all named variants, `Enum!` writes each payload in the shortest spelling of the same type.
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

// Property: a sum tagged by position keeps the plain list form, since a variant cannot be named by a position.
#[test]
fn a_sum_tagged_by_position_stays_a_plain_sum() {
    assert_eq!(
        resugar_lists("Either<Field<Index<0>, u8>, Void>").as_deref(),
        Some("Sum![Field<Index<0>, u8>]"),
    );
}

// Property: a list with any element that is not a `Field` keeps the plain list form.
#[test]
fn a_mixed_list_stays_a_plain_product() {
    // Not every element is a `Field`, so the list keeps its `Product!` form rather than `Struct!`.
    assert_eq!(
        resugar_lists("Cons<u64, Cons<Field<Symbol!(\"x\"), u8>, Nil>>").as_deref(),
        Some("Product![u64, Field<Symbol!(\"x\"), u8>]"),
    );
}

// Property: for all spines, each element is resugared recursively, so a nested spine resugars in turn.
#[test]
fn resugars_a_nested_list() {
    assert_eq!(
        resugar_lists("Cons<Either<u8, Void>, Nil>").as_deref(),
        Some("Product![Sum![u8]]"),
    );
}

// Property: for all text holding a spine, only the spine changes.
#[test]
fn resugars_a_spine_embedded_in_a_message() {
    assert_eq!(
        resugar_lists("required for `Cons<u64, Nil>` to implement `Foo`").as_deref(),
        Some("required for `Product![u64]` to implement `Foo`"),
    );
}

// Property: for all spines not closing on their exact terminator, `resugar_lists` gives `None`.
#[test]
fn declines_a_spine_that_does_not_terminate() {
    // The tail is not the exact `Nil`/`Void` terminator, so the structural match fails and the
    // text is left alone rather than mis-rewritten.
    assert_eq!(resugar_lists("Cons<u64, RestOfList>"), None);
    assert_eq!(resugar_lists("Either<u64, Either<f64, Nil>>"), None);
}

// Property: a cell name that only ends a longer identifier is not a spine cell.
#[test]
fn does_not_match_cons_inside_path_cons() {
    // `PathCons` ends in `Cons`, but its `Cons<` is not a standalone cell, so a `PathCons` spine is
    // left for the path resugaring rather than being read as a `Product!`.
    assert_eq!(resugar_lists("PathCons<Foo, Nil>"), None);
}
