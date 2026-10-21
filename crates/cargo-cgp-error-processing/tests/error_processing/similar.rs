//! Finding near misses: similar field names and similar wiring keys.

use cargo_cgp_error_processing::{similar_key, similar_name};

#[test]
fn a_transposed_or_recased_name_is_a_near_miss() {
    assert_eq!(
        similar_name("frist_name", ["last_name", "first_name"]),
        Some("first_name")
    );
    assert_eq!(similar_name("Height", ["width", "height"]), Some("height"));
}

#[test]
fn a_distant_or_identical_name_is_not_a_near_miss() {
    assert_eq!(similar_name("height", ["width"]), None);
    // Two names sharing a long stem stay apart: three edits are more than a slip.
    assert_eq!(similar_name("bar_provider", ["foo_provider"]), None);
    assert_eq!(similar_name("name", ["name"]), None);
}

#[test]
fn the_closest_name_wins() {
    assert_eq!(similar_name("widht", ["wide", "width"]), Some("width"));
}

#[test]
fn a_key_with_one_segment_recased_is_a_near_miss() {
    assert_eq!(
        similar_key("@app.GreeterComponent", ["@App.GreeterComponent"]),
        Some("@App.GreeterComponent")
    );
}

#[test]
fn a_key_with_one_symbol_misspelled_is_a_near_miss() {
    assert_eq!(
        similar_key("@app.GreeterComponent", ["@ap.GreeterComponent"]),
        Some("@ap.GreeterComponent")
    );
}

#[test]
fn a_key_with_a_segment_doubled_or_missing_is_a_near_miss() {
    assert_eq!(
        similar_key(
            "@app.ShowImplComponent.ShowImplComponent.String",
            ["@app.ShowImplComponent.String"]
        ),
        Some("@app.ShowImplComponent.String")
    );
    // A bare component looked up where the program wired it under a prefix.
    assert_eq!(
        similar_key("GreeterComponent", ["@app.GreeterComponent"]),
        Some("@app.GreeterComponent")
    );
}

#[test]
fn different_types_or_a_dropped_last_segment_are_not_near_misses() {
    // Two type segments name two different types, however close their spelling.
    assert_eq!(
        similar_key("BarProviderComponent", ["BazProviderComponent"]),
        None
    );
    assert_eq!(
        similar_key("@ItemEncoderComponent.u64", ["@ItemEncoderComponent.u32"]),
        None
    );
    // A primitive is a type segment too, not a symbol.
    assert_eq!(
        similar_key(
            "@VariantEncoderComponent.f64",
            ["@VariantEncoderComponent.u64"]
        ),
        None
    );
    // The last segment names what is looked up, so a key without it is another key.
    assert_eq!(similar_key("@app.GreeterComponent", ["@app"]), None);
    // Two segments more is two slips, not one.
    assert_eq!(
        similar_key("GreeterComponent", ["@app.inner.GreeterComponent"]),
        None
    );
    assert_eq!(
        similar_key("@app.GreeterComponent", ["@app.GreeterComponent"]),
        None
    );
}
