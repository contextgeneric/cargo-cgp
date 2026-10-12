//! Tests for the structural-mistake wording.
//!
//! `plan_structural_mistake` and `structural_mistake_help` are pure functions over the rustc-free
//! [`StructuralMistake`] model, so they are driven over hand-built mistakes; the driver fills the
//! same model in from the generated items an error lands on.

use cargo_cgp_error_processing::{
    StructuralMistake, plan_structural_mistake, structural_mistake_help,
};

#[test]
fn a_duplicate_check_names_the_entry_and_the_context() {
    let mistake = StructuralMistake::DuplicateCheck {
        context: "App".to_owned(),
        entry: "GreeterComponent".to_owned(),
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E015] `check_components!` checks `GreeterComponent` twice for `App`",
    );
    assert_eq!(
        structural_mistake_help(&mistake),
        "remove one of the two `GreeterComponent` entries",
    );
}

#[test]
fn a_duplicate_check_names_a_generic_entry_as_the_table_wrote_it() {
    let mistake = StructuralMistake::DuplicateCheck {
        context: "App".to_owned(),
        entry: "ShowerComponent: u32".to_owned(),
    };
    assert_eq!(
        plan_structural_mistake(&mistake),
        "[CGP-E015] `check_components!` checks `ShowerComponent: u32` twice for `App`",
    );
}
