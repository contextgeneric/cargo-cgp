//! One plan value for each class the emitter reshapes before the typed resolver.
//!
//! Each such class words its diagnostic through a header function and, for most, help and note
//! functions with their own return shapes (a `String`, an `Option`, or a `Vec`). A [`ClassPlan`]
//! gathers one class's wording into one value, so the emitter writes every reshape the same way.
//! Each constructor only composes the class's existing wording functions, which own the text and
//! its tests. The typed resolver's wording keeps its own [`DiagnosisPlan`](crate::DiagnosisPlan),
//! since its note is rendered at the flush rather than when the diagnostic arrives.

use crate::diagnosis::{
    CgpImplMisuse, MissingUseProvider, OrphanConflict, ProviderImplWithoutMacro,
    ProviderTraitInUses, StructuralMistake, UndeclaredRequirement, UndeclaredTrait,
    UnimportedAbstractType, UnresolvedWiringName, WiringConflict, cgp_impl_misuse_help,
    missing_use_provider_help, orphan_conflict_help, plan_cgp_impl_misuse,
    plan_missing_use_provider, plan_orphan_conflict, plan_provider_impl_without_macro,
    plan_provider_trait_in_uses, plan_structural_mistake, plan_undeclared_requirement,
    plan_undeclared_trait, plan_unimported_abstract_type, plan_unresolved_wiring_name,
    plan_wiring_conflict, provider_impl_without_macro_help, provider_trait_in_uses_help,
    structural_mistake_help, structural_mistake_note, undeclared_requirement_helps,
    undeclared_trait_help, unimported_abstract_type_help, unresolved_wiring_name_help,
    wiring_conflict_help,
};

/// The wording of one reshaped diagnostic: the coded header that replaces rustc's message, the
/// notes that say why the program fails, and the `help`s that name the fix. A class with no note,
/// or with no fix it can state, leaves that list empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassPlan {
    /// The coded header that replaces rustc's message.
    pub header: String,
    /// The notes that say why the program fails, in order.
    pub notes: Vec<String>,
    /// The `help`s that name the fix, in order.
    pub helps: Vec<String>,
}

impl ClassPlan {
    /// A duplicate-key wiring conflict (`[CGP-E010]` to `[CGP-E014]`); only a redirect collision
    /// that names its provider has a help.
    pub fn wiring_conflict(conflict: &WiringConflict) -> Self {
        Self {
            header: plan_wiring_conflict(conflict),
            notes: Vec::new(),
            helps: wiring_conflict_help(conflict).into_iter().collect(),
        }
    }

    /// An orphan-rule namespace registration (`[CGP-E022]`), with its ownership fix.
    pub fn orphan_conflict(conflict: &OrphanConflict) -> Self {
        Self {
            header: plan_orphan_conflict(conflict),
            notes: Vec::new(),
            helps: vec![orphan_conflict_help(conflict)],
        }
    }

    /// A structural mistake in a wiring or its checks, with its fix and, for the mistakes whose
    /// header does not say why the program fails, a note.
    pub fn structural_mistake(mistake: &StructuralMistake) -> Self {
        Self {
            header: plan_structural_mistake(mistake),
            notes: structural_mistake_note(mistake).into_iter().collect(),
            helps: vec![structural_mistake_help(mistake)],
        }
    }

    /// A trait a CGP body uses without declaring it (`[CGP-E030]`); the help is left out when the
    /// bound cannot be written.
    pub fn undeclared_trait(undeclared: &UndeclaredTrait) -> Self {
        Self {
            header: plan_undeclared_trait(undeclared),
            notes: Vec::new(),
            helps: undeclared_trait_help(undeclared).into_iter().collect(),
        }
    }

    /// An inner provider used without `#[use_provider]` (`[CGP-E031]`); the help is left out when
    /// the provider trait cannot be written.
    pub fn missing_use_provider(missing: &MissingUseProvider) -> Self {
        Self {
            header: plan_missing_use_provider(missing),
            notes: Vec::new(),
            helps: missing_use_provider_help(missing).into_iter().collect(),
        }
    }

    /// A `#[cgp_impl]` that names the wrong half of a component (`[CGP-E032]` to `[CGP-E034]`).
    pub fn cgp_impl_misuse(misuse: &CgpImplMisuse) -> Self {
        Self {
            header: plan_cgp_impl_misuse(misuse),
            notes: Vec::new(),
            helps: vec![cgp_impl_misuse_help(misuse)],
        }
    }

    /// A provider trait named in `#[uses]` (`[CGP-E037]`).
    pub fn provider_trait_in_uses(misuse: &ProviderTraitInUses) -> Self {
        Self {
            header: plan_provider_trait_in_uses(misuse),
            notes: Vec::new(),
            helps: vec![provider_trait_in_uses_help(misuse)],
        }
    }

    /// A provider-trait impl written without `#[cgp_provider]` (`[CGP-E039]`).
    pub fn provider_impl_without_macro(mistake: &ProviderImplWithoutMacro) -> Self {
        Self {
            header: plan_provider_impl_without_macro(mistake),
            notes: Vec::new(),
            helps: vec![provider_impl_without_macro_help(mistake)],
        }
    }

    /// An abstract type used without `#[use_type]` (`[CGP-E035]`).
    pub fn unimported_abstract_type(unimported: &UnimportedAbstractType) -> Self {
        Self {
            header: plan_unimported_abstract_type(unimported),
            notes: Vec::new(),
            helps: vec![unimported_abstract_type_help(unimported)],
        }
    }

    /// Requirements a provider does not declare (`[CGP-E036]`), with one help per kind of
    /// declaration they need.
    pub fn undeclared_requirement(undeclared: &UndeclaredRequirement) -> Self {
        Self {
            header: plan_undeclared_requirement(undeclared),
            notes: Vec::new(),
            helps: undeclared_requirement_helps(undeclared),
        }
    }

    /// A wiring name that does not resolve (`[CGP-E038]`, `[CGP-E040]` to `[CGP-E044]`); the help
    /// is left out when the tool knows no fix.
    pub fn unresolved_wiring_name(name: &UnresolvedWiringName) -> Self {
        Self {
            header: plan_unresolved_wiring_name(name),
            notes: Vec::new(),
            helps: unresolved_wiring_name_help(name).into_iter().collect(),
        }
    }
}
