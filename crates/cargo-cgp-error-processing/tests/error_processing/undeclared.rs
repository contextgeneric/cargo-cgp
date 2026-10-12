//! Tests for the used-but-undeclared trait wording (`[CGP-E030]`).
//!
//! `plan_undeclared_trait` and `undeclared_trait_help` are pure functions over the rustc-free
//! [`UndeclaredTrait`] model, so they are driven directly over hand-built values. The driver fills
//! the model in from the live `TyCtxt` when a `#[cgp_fn]` or `#[cgp_impl]` body calls a trait its
//! `#[uses(…)]` list omits.

use cargo_cgp_error_processing::{UndeclaredTrait, plan_undeclared_trait, undeclared_trait_help};

fn undeclared(trait_name: &str, bound: Option<&str>, unresolved: &[&str]) -> UndeclaredTrait {
    UndeclaredTrait {
        trait_name: trait_name.to_owned(),
        bound: bound.map(str::to_owned),
        unresolved: unresolved.iter().map(|name| (*name).to_owned()).collect(),
    }
}

#[test]
fn the_header_names_the_trait_not_the_generated_context() {
    assert_eq!(
        plan_undeclared_trait(&undeclared("GetCount", Some("GetCount"), &[])),
        "[CGP-E030] the trait `GetCount` is used but not declared as a dependency",
    );
}

#[test]
fn the_help_names_the_uses_attribute_to_write() {
    assert_eq!(
        undeclared_trait_help(&undeclared("GetCount", Some("GetCount"), &[])).as_deref(),
        Some("declare it as a dependency with `#[uses(GetCount)]`"),
    );
}

#[test]
fn a_generic_trait_is_declared_with_the_arguments_the_call_fixes() {
    let model = undeclared("CanFormatPair", Some("CanFormatPair<(u32, u64)>"), &[]);
    // The header names the trait alone; the help writes the bound with its arguments, since
    // `#[uses(CanFormatPair)]` would not compile.
    assert_eq!(
        plan_undeclared_trait(&model),
        "[CGP-E030] the trait `CanFormatPair` is used but not declared as a dependency",
    );
    assert_eq!(
        undeclared_trait_help(&model).as_deref(),
        Some("declare it as a dependency with `#[uses(CanFormatPair<(u32, u64)>)]`"),
    );
}

#[test]
fn a_parameter_the_call_does_not_give_away_is_left_for_the_reader() {
    let one = undeclared(
        "CanRaiseError",
        Some("CanRaiseError<SourceError>"),
        &["SourceError"],
    );
    assert_eq!(
        undeclared_trait_help(&one).as_deref(),
        Some(
            "declare it as a dependency with `#[uses(CanRaiseError<SourceError>)]`, with the type \
             the call uses in place of `SourceError`"
        ),
    );
    let two = undeclared("CanPair", Some("CanPair<A, B>"), &["A", "B"]);
    assert_eq!(
        undeclared_trait_help(&two).as_deref(),
        Some(
            "declare it as a dependency with `#[uses(CanPair<A, B>)]`, with the types the call \
             uses in place of `A` and `B`"
        ),
    );
}

#[test]
fn no_help_is_given_when_the_bound_cannot_be_written() {
    assert_eq!(
        undeclared_trait_help(&undeclared("HasReference", None, &[])),
        None
    );
}
