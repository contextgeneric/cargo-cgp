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
    CgpImplMisuse, CheckForm, CheckTraitName, ClassPlan, MissingUseProvider, OrphanConflict,
    OrphanTrigger, ProviderImplWithoutMacro, ProviderTraitInUses, Requirement, StructuralMistake,
    UndeclaredRequirement, UndeclaredTrait, UnimportedAbstractType, UnresolvedWiringName,
    WiringConflict, WiringKey,
};

/// Check `plan`, the row named `case`, against the property above: its header carries `code`, and
/// it holds `notes` notes and `helps` helps, none empty and none a coded header.
fn check_plan(case: &str, plan: &ClassPlan, code: &str, notes: usize, helps: usize) {
    assert!(
        plan.header.starts_with(&format!("[{code}] ")),
        "{case}: the header carries the class code {code}: {:?}",
        plan.header,
    );
    assert_eq!(
        plan.notes.len(),
        notes,
        "{case}: the class's notes: {:?}",
        plan.notes
    );
    assert_eq!(
        plan.helps.len(),
        helps,
        "{case}: the class's helps: {:?}",
        plan.helps
    );
    for text in plan.notes.iter().chain(&plan.helps) {
        assert!(
            !text.is_empty() && !text.starts_with("[CGP-"),
            "{case}: a note or help is a sentence of its own, not a header: {text:?}",
        );
    }
}

/// One row of the table: the case, its plan, the class code, and the notes and helps the catalog
/// gives that case.
type Row = (&'static str, ClassPlan, &'static str, usize, usize);

fn wiring_rows() -> Vec<Row> {
    let redirect = |provider: Option<&str>| WiringConflict::Redirect {
        context: "Shop".to_owned(),
        key: WiringKey::Component("PricerComponent".to_owned()),
        path: "@shop.PricerComponent".to_owned(),
        provider: provider.map(str::to_owned),
        params: Vec::new(),
    };
    let duplicate = WiringConflict::Duplicate {
        context: "Shop".to_owned(),
        key: WiringKey::Component("PricerComponent".to_owned()),
    };
    let orphan = OrphanConflict {
        namespace: "UpstreamNamespace".to_owned(),
        key: WiringKey::Component("PricerComponent".to_owned()),
        trigger: OrphanTrigger::Register,
    };
    vec![
        // The catalog's fix for `CGP-E010` is the carets themselves, so no help is stated.
        (
            "a duplicate key",
            ClassPlan::wiring_conflict(&duplicate),
            "CGP-E010",
            0,
            0,
        ),
        (
            "a redirect collision naming its provider",
            ClassPlan::wiring_conflict(&redirect(Some("FixedPrice"))),
            "CGP-E013",
            0,
            1,
        ),
        // Concern: an absent fix could surface as an empty help.
        // Handled: the optional help is collected from an `Option`, so `None` adds no entry.
        (
            "a redirect collision naming no provider",
            ClassPlan::wiring_conflict(&redirect(None)),
            "CGP-E013",
            0,
            0,
        ),
        (
            "an orphan registration",
            ClassPlan::orphan_conflict(&orphan),
            "CGP-E022",
            0,
            1,
        ),
    ]
}

fn structural_rows() -> Vec<Row> {
    let check_trait =
        |name: CheckTraitName, first: CheckForm| StructuralMistake::DuplicateCheckTrait {
            name,
            first,
            second: CheckForm::CheckComponents,
        };
    let derived = CheckTraitName::Derived {
        context: "Shop".to_owned(),
    };
    let given = || CheckTraitName::Given {
        check_trait: "CheckShop".to_owned(),
    };
    let duplicate = StructuralMistake::DuplicateCheck {
        context: "Shop".to_owned(),
        entry: "PricerComponent".to_owned(),
    };
    let shared = StructuralMistake::SharedDispatchMethod {
        method: "price".to_owned(),
        helper: "__compute_price__".to_owned(),
        computer: "ComputePrice".to_owned(),
    };
    vec![
        (
            "a duplicate check",
            ClassPlan::structural_mistake(&duplicate),
            "CGP-E015",
            0,
            1,
        ),
        (
            "two tables deriving one check trait",
            ClassPlan::structural_mistake(&check_trait(derived, CheckForm::CheckComponents)),
            "CGP-E016",
            0,
            1,
        ),
        (
            "two tables of one macro given one check-trait name",
            ClassPlan::structural_mistake(&check_trait(given(), CheckForm::CheckComponents)),
            "CGP-E016",
            0,
            1,
        ),
        (
            "tables of two macros given one check-trait name",
            ClassPlan::structural_mistake(&check_trait(
                given(),
                CheckForm::DelegateAndCheckComponents,
            )),
            "CGP-E016",
            0,
            1,
        ),
        // The catalog gives `CGP-E061` a note naming the two generated items, beside its fix.
        (
            "a shared dispatch method",
            ClassPlan::structural_mistake(&shared),
            "CGP-E061",
            1,
            1,
        ),
    ]
}

fn lowering_rows() -> Vec<Row> {
    let undeclared = |bound: Option<&str>| UndeclaredTrait {
        trait_name: "CanPrice".to_owned(),
        bound: bound.map(str::to_owned),
        unresolved: Vec::new(),
    };
    let missing = |provider_trait: Option<&str>| MissingUseProvider {
        inner: "InnerPricer".to_owned(),
        provider_trait: provider_trait.map(str::to_owned),
        unresolved: Vec::new(),
    };
    let consumer = CgpImplMisuse::ConsumerTrait {
        consumer: "CanPrice".to_owned(),
        provider: "Pricer".to_owned(),
        arguments: Vec::new(),
    };
    let non_cgp = CgpImplMisuse::NonCgpTrait {
        trait_name: "Display".to_owned(),
    };
    let bound = CgpImplMisuse::ConsumerProviderBound {
        consumer: "CanPrice".to_owned(),
        provider: "Pricer".to_owned(),
        arguments: Vec::new(),
    };
    let in_uses = ProviderTraitInUses {
        provider: "Pricer".to_owned(),
        consumer: "CanPrice".to_owned(),
        arguments: Vec::new(),
    };
    let without_macro = ProviderImplWithoutMacro {
        provider: "FixedPrice".to_owned(),
        provider_trait: "Pricer".to_owned(),
    };
    let unimported = UnimportedAbstractType {
        type_name: "Currency".to_owned(),
        trait_name: "HasCurrencyType".to_owned(),
    };
    let requirement = |bound: &str, import: Option<&str>| Requirement {
        bound: bound.to_owned(),
        import: import.map(str::to_owned),
    };
    let requirements = |requirements: Vec<Requirement>| UndeclaredRequirement {
        provider_trait: "Pricer".to_owned(),
        provider: "FixedPrice".to_owned(),
        requirements,
    };
    let both = requirements(vec![
        requirement("HasTaxRate", None),
        requirement("HasCurrencyType", Some("HasCurrencyType.Currency")),
    ]);
    let uses_only = requirements(vec![
        requirement("HasTaxRate", None),
        requirement("HasDiscount", None),
    ]);
    vec![
        (
            "an undeclared trait with a writable bound",
            ClassPlan::undeclared_trait(&undeclared(Some("CanPrice"))),
            "CGP-E030",
            0,
            1,
        ),
        (
            "an undeclared trait without a writable bound",
            ClassPlan::undeclared_trait(&undeclared(None)),
            "CGP-E030",
            0,
            0,
        ),
        (
            "a missing use_provider with a writable trait",
            ClassPlan::missing_use_provider(&missing(Some("Pricer"))),
            "CGP-E031",
            0,
            1,
        ),
        (
            "a missing use_provider without a writable trait",
            ClassPlan::missing_use_provider(&missing(None)),
            "CGP-E031",
            0,
            0,
        ),
        (
            "a consumer trait in a provider impl",
            ClassPlan::cgp_impl_misuse(&consumer),
            "CGP-E032",
            0,
            1,
        ),
        (
            "cgp_impl on a non-CGP trait",
            ClassPlan::cgp_impl_misuse(&non_cgp),
            "CGP-E033",
            0,
            1,
        ),
        (
            "a consumer trait in an inner-provider bound",
            ClassPlan::cgp_impl_misuse(&bound),
            "CGP-E034",
            0,
            1,
        ),
        (
            "an unimported abstract type",
            ClassPlan::unimported_abstract_type(&unimported),
            "CGP-E035",
            0,
            1,
        ),
        // The catalog gives `CGP-E036` a help for each kind of declaration: `#[uses]` for a trait
        // and `#[use_type]` for an abstract type.
        (
            "an undeclared trait and abstract type",
            ClassPlan::undeclared_requirement(&both),
            "CGP-E036",
            0,
            2,
        ),
        (
            "undeclared traits alone",
            ClassPlan::undeclared_requirement(&uses_only),
            "CGP-E036",
            0,
            1,
        ),
        (
            "a provider trait in uses",
            ClassPlan::provider_trait_in_uses(&in_uses),
            "CGP-E037",
            0,
            1,
        ),
        (
            "a provider impl without its macro",
            ClassPlan::provider_impl_without_macro(&without_macro),
            "CGP-E039",
            0,
            1,
        ),
    ]
}

fn wiring_name_rows() -> Vec<Row> {
    let consumer_key = UnresolvedWiringName::ConsumerTraitAsKey {
        consumer: "CanPrice".to_owned(),
        component: "PricerComponent".to_owned(),
    };
    let no_near_miss = UnresolvedWiringName::ProviderNotDeclared {
        provider: "FixedPrise".to_owned(),
        key: "PricerComponent".to_owned(),
        near_miss: None,
    };
    let undeclared_provider = UnresolvedWiringName::CgpImplProviderNotDeclared {
        provider: "FixedPrice".to_owned(),
    };
    let nested_type = UnresolvedWiringName::PathTypeNotDeclared {
        name: "Money".to_owned(),
        whole_segment: false,
    };
    vec![
        (
            "a consumer trait as a key",
            ClassPlan::unresolved_wiring_name(&consumer_key),
            "CGP-E040",
            0,
            1,
        ),
        // The catalog gives `CGP-E043` a help only for a near miss.
        (
            "an undeclared provider without a near miss",
            ClassPlan::unresolved_wiring_name(&no_near_miss),
            "CGP-E043",
            0,
            0,
        ),
        (
            "an undeclared cgp_impl provider",
            ClassPlan::unresolved_wiring_name(&undeclared_provider),
            "CGP-E038",
            0,
            1,
        ),
        // The catalog gives `CGP-E044` a help only for a whole segment of the path.
        (
            "a type nested in a path segment",
            ClassPlan::unresolved_wiring_name(&nested_type),
            "CGP-E044",
            0,
            0,
        ),
    ]
}

// Property: the module's property, over one row per case the catalog gives its own help count.
#[test]
fn each_class_plan_holds_the_helps_the_catalog_gives_it() {
    for (case, plan, code, notes, helps) in wiring_rows()
        .into_iter()
        .chain(structural_rows())
        .chain(lowering_rows())
        .chain(wiring_name_rows())
    {
        check_plan(case, &plan, code, notes, helps);
    }
}
