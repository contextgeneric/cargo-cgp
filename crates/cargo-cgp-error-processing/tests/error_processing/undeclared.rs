//! Tests for the used-but-undeclared trait wording (`[CGP-E030]`).
//!
//! `plan_undeclared_trait` and `undeclared_trait_help` are pure functions over the rustc-free
//! [`UndeclaredTrait`] model, so they are driven directly over hand-built values. The driver fills
//! the model in from the live `TyCtxt` when a `#[cgp_fn]` or `#[cgp_impl]` body calls a trait its
//! `#[uses(…)]` list omits.

use cargo_cgp_error_processing::{UndeclaredTrait, plan_undeclared_trait, undeclared_trait_help};

fn undeclared(trait_name: &str) -> UndeclaredTrait {
    UndeclaredTrait {
        trait_name: trait_name.to_owned(),
    }
}

#[test]
fn the_header_names_the_trait_not_the_generated_context() {
    assert_eq!(
        plan_undeclared_trait(&undeclared("GetCount")),
        "[CGP-E030] the trait `GetCount` is used but not declared as a dependency",
    );
}

#[test]
fn the_help_names_the_uses_attribute_to_write() {
    assert_eq!(
        undeclared_trait_help(&undeclared("GetCount")),
        "declare it as a dependency with `#[uses(GetCount)]`",
    );
}

#[test]
fn the_trait_name_is_used_verbatim_in_both() {
    // The wording adds nothing to the name the driver supplies, so the header and the help always
    // name the same trait.
    let name = "HasRectangleFields";
    let model = undeclared(name);
    assert!(plan_undeclared_trait(&model).contains(&format!("`{name}`")));
    assert!(undeclared_trait_help(&model).contains(&format!("#[uses({name})]")));
}
