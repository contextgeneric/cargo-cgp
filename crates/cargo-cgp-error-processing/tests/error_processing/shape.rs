//! Tests for the shared `Struct!`/`Enum!` spelling rules.
//!
//! The text post-processing and the driver's typed renderer both call these functions, so each
//! rule is pinned once here, over names and already-rendered values, with no compiler involved.

use cargo_cgp_error_processing::{
    ShapeTag, render_enum_shape, render_struct_shape, render_variant, shape_name,
};

fn name(value: &str) -> ShapeTag {
    ShapeTag::Name(value.to_owned())
}

fn fields(entries: &[(ShapeTag, &str)]) -> Vec<(ShapeTag, String)> {
    entries
        .iter()
        .map(|(tag, value)| (tag.clone(), (*value).to_owned()))
        .collect()
}

#[test]
fn an_identifier_is_written_as_is() {
    assert_eq!(shape_name("width").as_deref(), Some("width"));
    assert_eq!(shape_name("_private").as_deref(), Some("_private"));
    assert_eq!(shape_name("Circle").as_deref(), Some("Circle"));
    assert_eq!(shape_name("größe").as_deref(), Some("größe"));
}

#[test]
fn a_keyword_is_written_raw() {
    assert_eq!(shape_name("type").as_deref(), Some("r#type"));
    assert_eq!(shape_name("match").as_deref(), Some("r#match"));
    assert_eq!(shape_name("gen").as_deref(), Some("r#gen"));
}

#[test]
fn a_name_no_body_can_write_has_no_spelling() {
    for unwritable in [
        "", "foo bar", "0", "1st", "a-b", "_", "self", "Self", "super", "crate",
    ] {
        assert_eq!(shape_name(unwritable), None, "{unwritable:?}");
    }
}

#[test]
fn named_cells_render_as_a_named_struct() {
    assert_eq!(
        render_struct_shape(&fields(&[(name("width"), "f64"), (name("type"), "u8")])).as_deref(),
        Some("Struct! { width: f64, r#type: u8 }"),
    );
}

#[test]
fn index_cells_in_order_render_as_a_tuple_struct() {
    assert_eq!(
        render_struct_shape(&fields(&[
            (ShapeTag::Index(0), "u64"),
            (ShapeTag::Index(1), "String"),
        ]))
        .as_deref(),
        Some("Struct!(u64, String)"),
    );
}

#[test]
fn a_product_with_no_struct_spelling_declines() {
    // `Struct!(T)` is the bare `T`, so a single `Index` cell has no spelling.
    assert_eq!(
        render_struct_shape(&fields(&[(ShapeTag::Index(0), "u64")])),
        None
    );
    // Positions must count up from 0 with no gap.
    assert_eq!(
        render_struct_shape(&fields(&[
            (ShapeTag::Index(1), "u8"),
            (ShapeTag::Index(0), "u8")
        ])),
        None,
    );
    assert_eq!(
        render_struct_shape(&fields(&[
            (ShapeTag::Index(0), "u8"),
            (ShapeTag::Index(2), "u8")
        ])),
        None,
    );
    // A body cannot mix the named and positional forms.
    assert_eq!(
        render_struct_shape(&fields(&[(name("a"), "u8"), (ShapeTag::Index(1), "u8")])),
        None,
    );
    // A name that is not an identifier cannot be written.
    assert_eq!(
        render_struct_shape(&fields(&[(name("foo bar"), "u8")])),
        None
    );
    assert_eq!(render_struct_shape(&[]), None);
}

#[test]
fn a_variant_takes_the_shortest_spelling_of_its_payload() {
    assert_eq!(render_variant("Empty", None), "Empty");
    assert_eq!(render_variant("Circle", Some("f64")), "Circle(f64)");
    assert_eq!(
        render_variant("Rect", Some("Struct! { width: f64, height: f64 }")),
        "Rect { width: f64, height: f64 }",
    );
    assert_eq!(
        render_variant("Pair", Some("Struct!(u8, u16)")),
        "Pair(u8, u16)"
    );
    // A payload whose list has no `Struct!` spelling stays the single positional field.
    assert_eq!(
        render_variant("One", Some("Product![Field<Index<0>, u8>]")),
        "One(Product![Field<Index<0>, u8>])",
    );
}

#[test]
fn a_variant_payload_that_only_starts_with_a_shape_stays_positional() {
    // The shape is not the whole payload, so its braces are not the variant's own.
    assert_eq!(
        render_variant("V", Some("Struct!(u8, u16)::Assoc")),
        "V(Struct!(u8, u16)::Assoc)",
    );
    assert_eq!(
        render_variant("V", Some("Struct! { a: u8 } }{ b: u8 }")),
        "V(Struct! { a: u8 } }{ b: u8 })",
    );
}

#[test]
fn named_variants_render_as_an_enum() {
    assert_eq!(
        render_enum_shape(&[
            (name("Empty"), None),
            (name("Circle"), Some("f64".to_owned())),
            (name("Rect"), Some("Struct! { width: f64 }".to_owned())),
            (name("match"), Some("Struct!(u8, u16)".to_owned())),
        ])
        .as_deref(),
        Some("Enum! { Empty, Circle(f64), Rect { width: f64 }, r#match(u8, u16) }"),
    );
}

#[test]
fn a_sum_with_no_enum_spelling_declines() {
    assert_eq!(
        render_enum_shape(&[(ShapeTag::Index(0), Some("u8".to_owned()))]),
        None
    );
    assert_eq!(render_enum_shape(&[(name("not a name"), None)]), None);
    assert_eq!(render_enum_shape(&[]), None);
}
