//! Tests for [`ClassPlan`], the one value each reshaped class's wording is gathered into.
//!
//! Each constructor composes the class's wording functions, whose text the class's own test
//! module pins. These tests check what the composition adds: the header, the notes, and the helps
//! each land in their own list, with a help exactly where the error-code catalog
//! (`cgp-knowledge-base/cargo-cgp/error-code.md`) says the class states a fix.
//!
//! Property: for all models m of a class C, `ClassPlan::c(&m).header` starts with C's code as the
//!   catalog numbers it, `notes` holds C's explanations, and `helps` holds one entry per fix the
//!   catalog says C states for m, each a non-empty sentence that is not itself a coded header.
//! Relies on: the wording tests of each class (`wiring.rs`, `orphan.rs`, `structural.rs`,
//!   `undeclared.rs`, `cgp_impl_misuse.rs`, `macro_less_provider.rs`, `unimported.rs`,
//!   `requirement.rs`, and `wiring_name.rs`), which pin the text itself.

use cargo_cgp_error_processing::{
    CgpImplMisuse, ClassPlan, MissingUseProvider, OrphanConflict, OrphanTrigger,
    ProviderImplWithoutMacro, ProviderTraitInUses, Requirement, StructuralMistake,
    UndeclaredRequirement, UndeclaredTrait, UnimportedAbstractType, UnresolvedWiringName,
    WiringConflict, WiringKey,
};

/// Check `plan` against the property above: its header carries `code`, and it holds `notes` notes
/// and `helps` helps, none empty and none a coded header.
fn check_plan(plan: &ClassPlan, code: &str, notes: usize, helps: usize) {
    assert!(
        plan.header.starts_with(&format!("[{code}] ")),
        "the header carries the class code {code}: {:?}",
        plan.header,
    );
    assert_eq!(
        plan.notes.len(),
        notes,
        "the class's notes: {:?}",
        plan.notes
    );
    assert_eq!(
        plan.helps.len(),
        helps,
        "the class's helps: {:?}",
        plan.helps
    );
    for text in plan.notes.iter().chain(&plan.helps) {
        assert!(
            !text.is_empty() && !text.starts_with("[CGP-"),
            "a note or help is a sentence of its own, not a header: {text:?}",
        );
    }
}

#[test]
fn a_duplicate_key_has_no_help() {
    // The catalog's fix for `CGP-E010` is the carets themselves, so no help is stated.
    let conflict = WiringConflict::Duplicate {
        context: "Shop".to_owned(),
        key: WiringKey::Component("PricerComponent".to_owned()),
    };
    check_plan(&ClassPlan::wiring_conflict(&conflict), "CGP-E010", 0, 0);
}

#[test]
fn a_redirect_collision_naming_its_provider_has_a_help() {
    let conflict = WiringConflict::Redirect {
        context: "Shop".to_owned(),
        key: WiringKey::Component("PricerComponent".to_owned()),
        path: "@shop.PricerComponent".to_owned(),
        provider: Some("FixedPrice".to_owned()),
        params: Vec::new(),
    };
    check_plan(&ClassPlan::wiring_conflict(&conflict), "CGP-E013", 0, 1);
}

#[test]
fn a_redirect_collision_naming_no_provider_has_no_help() {
    // Concern: an absent fix could surface as an empty help.
    // Handled: the optional help is collected from an `Option`, so `None` adds no entry.
    let conflict = WiringConflict::Redirect {
        context: "Shop".to_owned(),
        key: WiringKey::Component("PricerComponent".to_owned()),
        path: "@shop.PricerComponent".to_owned(),
        provider: None,
        params: Vec::new(),
    };
    check_plan(&ClassPlan::wiring_conflict(&conflict), "CGP-E013", 0, 0);
}

#[test]
fn an_orphan_registration_has_its_ownership_help() {
    let conflict = OrphanConflict {
        namespace: "UpstreamNamespace".to_owned(),
        key: WiringKey::Component("PricerComponent".to_owned()),
        trigger: OrphanTrigger::Register,
    };
    check_plan(&ClassPlan::orphan_conflict(&conflict), "CGP-E022", 0, 1);
}

#[test]
fn a_duplicate_check_has_a_help_and_no_note() {
    let mistake = StructuralMistake::DuplicateCheck {
        context: "Shop".to_owned(),
        entry: "PricerComponent".to_owned(),
    };
    check_plan(&ClassPlan::structural_mistake(&mistake), "CGP-E015", 0, 1);
}

#[test]
fn a_shared_dispatch_method_has_a_note_and_a_help() {
    // The catalog gives `CGP-E061` a note naming the two generated items, beside its fix.
    let mistake = StructuralMistake::SharedDispatchMethod {
        method: "price".to_owned(),
        helper: "__compute_price__".to_owned(),
        computer: "ComputePrice".to_owned(),
    };
    check_plan(&ClassPlan::structural_mistake(&mistake), "CGP-E061", 1, 1);
}

#[test]
fn an_undeclared_trait_with_a_writable_bound_has_a_help() {
    let undeclared = UndeclaredTrait {
        trait_name: "CanPrice".to_owned(),
        bound: Some("CanPrice".to_owned()),
        unresolved: Vec::new(),
    };
    check_plan(&ClassPlan::undeclared_trait(&undeclared), "CGP-E030", 0, 1);
}

#[test]
fn an_undeclared_trait_without_a_writable_bound_has_no_help() {
    let undeclared = UndeclaredTrait {
        trait_name: "CanPrice".to_owned(),
        bound: None,
        unresolved: Vec::new(),
    };
    check_plan(&ClassPlan::undeclared_trait(&undeclared), "CGP-E030", 0, 0);
}

#[test]
fn a_missing_use_provider_with_a_writable_trait_has_a_help() {
    let missing = MissingUseProvider {
        inner: "InnerPricer".to_owned(),
        provider_trait: Some("Pricer".to_owned()),
        unresolved: Vec::new(),
    };
    check_plan(&ClassPlan::missing_use_provider(&missing), "CGP-E031", 0, 1);
}

#[test]
fn a_missing_use_provider_without_a_writable_trait_has_no_help() {
    let missing = MissingUseProvider {
        inner: "InnerPricer".to_owned(),
        provider_trait: None,
        unresolved: Vec::new(),
    };
    check_plan(&ClassPlan::missing_use_provider(&missing), "CGP-E031", 0, 0);
}

#[test]
fn each_wrong_component_half_has_a_help() {
    let consumer = CgpImplMisuse::ConsumerTrait {
        consumer: "CanPrice".to_owned(),
        provider: "Pricer".to_owned(),
    };
    check_plan(&ClassPlan::cgp_impl_misuse(&consumer), "CGP-E032", 0, 1);
    let non_cgp = CgpImplMisuse::NonCgpTrait {
        trait_name: "Display".to_owned(),
    };
    check_plan(&ClassPlan::cgp_impl_misuse(&non_cgp), "CGP-E033", 0, 1);
    let bound = CgpImplMisuse::ConsumerProviderBound {
        consumer: "CanPrice".to_owned(),
        provider: "Pricer".to_owned(),
    };
    check_plan(&ClassPlan::cgp_impl_misuse(&bound), "CGP-E034", 0, 1);
}

#[test]
fn a_provider_trait_in_uses_has_a_help() {
    let misuse = ProviderTraitInUses {
        provider: "Pricer".to_owned(),
        consumer: "CanPrice".to_owned(),
    };
    check_plan(
        &ClassPlan::provider_trait_in_uses(&misuse),
        "CGP-E037",
        0,
        1,
    );
}

#[test]
fn a_provider_impl_without_its_macro_has_a_help() {
    let mistake = ProviderImplWithoutMacro {
        provider: "FixedPrice".to_owned(),
        provider_trait: "Pricer".to_owned(),
    };
    check_plan(
        &ClassPlan::provider_impl_without_macro(&mistake),
        "CGP-E039",
        0,
        1,
    );
}

#[test]
fn an_unimported_abstract_type_has_its_import_help() {
    let unimported = UnimportedAbstractType {
        type_name: "Currency".to_owned(),
        trait_name: "HasCurrencyType".to_owned(),
    };
    check_plan(
        &ClassPlan::unimported_abstract_type(&unimported),
        "CGP-E035",
        0,
        1,
    );
}

#[test]
fn an_undeclared_requirement_has_one_help_per_kind_of_declaration() {
    // The catalog gives `CGP-E036` a help for each kind of declaration: `#[uses]` for a trait and
    // `#[use_type]` for an abstract type.
    let both = UndeclaredRequirement {
        provider_trait: "Pricer".to_owned(),
        provider: "FixedPrice".to_owned(),
        requirements: vec![
            Requirement {
                bound: "HasTaxRate".to_owned(),
                import: None,
            },
            Requirement {
                bound: "HasCurrencyType".to_owned(),
                import: Some("HasCurrencyType.Currency".to_owned()),
            },
        ],
    };
    check_plan(&ClassPlan::undeclared_requirement(&both), "CGP-E036", 0, 2);
    let uses_only = UndeclaredRequirement {
        requirements: vec![
            Requirement {
                bound: "HasTaxRate".to_owned(),
                import: None,
            },
            Requirement {
                bound: "HasDiscount".to_owned(),
                import: None,
            },
        ],
        ..both
    };
    check_plan(
        &ClassPlan::undeclared_requirement(&uses_only),
        "CGP-E036",
        0,
        1,
    );
}

#[test]
fn an_unresolved_wiring_name_has_a_help_only_where_a_fix_is_known() {
    let consumer_key = UnresolvedWiringName::ConsumerTraitAsKey {
        consumer: "CanPrice".to_owned(),
        component: "PricerComponent".to_owned(),
    };
    check_plan(
        &ClassPlan::unresolved_wiring_name(&consumer_key),
        "CGP-E040",
        0,
        1,
    );
    // The catalog gives `CGP-E043` a help only for a near miss.
    let no_near_miss = UnresolvedWiringName::ProviderNotDeclared {
        provider: "FixedPrise".to_owned(),
        key: "PricerComponent".to_owned(),
        near_miss: None,
    };
    check_plan(
        &ClassPlan::unresolved_wiring_name(&no_near_miss),
        "CGP-E043",
        0,
        0,
    );
    let undeclared_provider = UnresolvedWiringName::CgpImplProviderNotDeclared {
        provider: "FixedPrice".to_owned(),
    };
    check_plan(
        &ClassPlan::unresolved_wiring_name(&undeclared_provider),
        "CGP-E038",
        0,
        1,
    );
    // The catalog gives `CGP-E044` a help only for a whole segment of the path.
    let nested_type = UnresolvedWiringName::PathTypeNotDeclared {
        name: "Money".to_owned(),
        whole_segment: false,
    };
    check_plan(
        &ClassPlan::unresolved_wiring_name(&nested_type),
        "CGP-E044",
        0,
        0,
    );
}
