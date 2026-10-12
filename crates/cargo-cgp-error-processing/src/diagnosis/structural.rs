//! The rustc-free model and wording for the structural mistakes in a wiring or its checks.
//!
//! Some mistakes are in the shape of what the macros were given rather than in what the wiring
//! resolves to: a check table that lists one entry twice, or two tables whose check traits share a
//! name. The compiler rejects the code the
//! macro generated from it with a bare coherence or name error about generated items (`__CheckApp`),
//! which names neither the table nor the entry. The driver recovers the mistake from the generated
//! items and fills [`StructuralMistake`] in; [`plan_structural_mistake`] words the header and
//! [`structural_mistake_help`] the fix.

use crate::code::{DUPLICATE_CHECK, DUPLICATE_CHECK_TRAIT};

/// The label on the first of two `check_components!` tables whose check traits share a name.
pub const FIRST_CHECK_TABLE_LABEL: &str = "the first table";

/// The label on the second of two `check_components!` tables whose check traits share a name, the
/// one the compiler reports.
pub const SECOND_CHECK_TABLE_LABEL: &str = "the second table";

/// A structural mistake the driver recovered from the generated items an error lands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralMistake {
    /// One `check_components!` table checks `entry` (a component, with its parameters when it
    /// takes any, as the table wrote them) twice for `context`.
    DuplicateCheck { context: String, entry: String },
    /// Two `check_components!` tables in one module generate check traits of the same name, either
    /// both derived from contexts whose paths end in `context` (`__Check{context}`) or both named
    /// `check_trait` by a `#[check_trait(...)]` attribute.
    DuplicateCheckTrait { name: CheckTraitName },
}

/// The name two `check_components!` tables gave their check traits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckTraitName {
    /// The name the macro derives, `__Check{context}`, from the last segment of the context's path.
    Derived { context: String },
    /// A name a `#[check_trait(...)]` attribute gave.
    Given { check_trait: String },
}

/// The header that replaces the compiler's message for a structural mistake.
pub fn plan_structural_mistake(mistake: &StructuralMistake) -> String {
    match mistake {
        StructuralMistake::DuplicateCheck { context, entry } => format!(
            "[{DUPLICATE_CHECK}] `check_components!` checks `{entry}` twice for `{context}`"
        ),
        StructuralMistake::DuplicateCheckTrait {
            name: CheckTraitName::Derived { context },
        } => format!(
            "[{DUPLICATE_CHECK_TRAIT}] two `check_components!` tables in one module check `{context}`"
        ),
        StructuralMistake::DuplicateCheckTrait {
            name: CheckTraitName::Given { check_trait },
        } => format!(
            "[{DUPLICATE_CHECK_TRAIT}] two `check_components!` tables in one module name their check trait `{check_trait}`"
        ),
    }
}

/// The `help` naming the fix for a structural mistake.
pub fn structural_mistake_help(mistake: &StructuralMistake) -> String {
    match mistake {
        StructuralMistake::DuplicateCheck { entry, .. } => {
            format!("remove one of the two `{entry}` entries")
        }
        StructuralMistake::DuplicateCheckTrait {
            name: CheckTraitName::Derived { .. },
        } => "merge the two tables, or name one table's check trait with `#[check_trait(...)]`"
            .to_owned(),
        StructuralMistake::DuplicateCheckTrait {
            name: CheckTraitName::Given { .. },
        } => "merge the two tables, or give one of them another `#[check_trait(...)]` name"
            .to_owned(),
    }
}
