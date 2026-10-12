//! Tests for the unimported abstract-type wording (`[CGP-E035]`).
//!
//! `plan_unimported_abstract_type` and `unimported_abstract_type_help` are pure functions over the
//! rustc-free [`UnimportedAbstractType`] model, driven here over hand-built values. The driver fills
//! the model in when a CGP definition names an abstract type it never imports.

use cargo_cgp_error_processing::{
    UnimportedAbstractType, plan_unimported_abstract_type, unimported_abstract_type_help,
};

fn unimported() -> UnimportedAbstractType {
    UnimportedAbstractType {
        type_name: "Error".to_owned(),
        trait_name: "HasErrorType".to_owned(),
    }
}

#[test]
fn the_header_names_the_abstract_type() {
    assert_eq!(
        plan_unimported_abstract_type(&unimported()),
        "[CGP-E035] the abstract type `Error` is used but not imported",
    );
}

#[test]
fn the_help_names_the_import_to_write() {
    assert_eq!(
        unimported_abstract_type_help(&unimported()),
        "import it with `#[use_type(HasErrorType.Error)]`",
    );
}
