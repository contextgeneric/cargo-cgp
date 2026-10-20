//! Tests for the help on a `#[use_type]` import that misspells its associated type.

use cargo_cgp_error_processing::{UseTypeMisspelling, use_type_misspelling_help};

fn misspelling(alias_is_name: bool) -> UseTypeMisspelling {
    UseTypeMisspelling {
        type_name: "Scalr".to_owned(),
        trait_name: "HasScalarType".to_owned(),
        similar: "Scalar".to_owned(),
        alias_is_name,
    }
}

#[test]
fn a_misspelled_alias_is_renamed_in_the_import_and_each_use() {
    assert_eq!(
        use_type_misspelling_help(&misspelling(true)),
        "rename `Scalr` to `Scalar` in the `#[use_type]` import from `HasScalarType` and in each use of `Scalr`",
    );
}

#[test]
fn an_as_alias_is_renamed_in_the_import_alone() {
    assert_eq!(
        use_type_misspelling_help(&misspelling(false)),
        "rename `Scalr` to `Scalar` in the `#[use_type]` import from `HasScalarType`",
    );
}
