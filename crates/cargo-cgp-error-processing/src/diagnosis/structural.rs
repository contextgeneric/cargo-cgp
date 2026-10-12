//! The rustc-free model and wording for the structural mistakes in a wiring or its checks.
//!
//! Some mistakes are in the shape of what the macros were given rather than in what the wiring
//! resolves to: a check table that lists one entry twice, say. The compiler rejects the code the
//! macro generated from it with a bare coherence or name error about generated items (`__CheckApp`),
//! which names neither the table nor the entry. The driver recovers the mistake from the generated
//! items and fills [`StructuralMistake`] in; [`plan_structural_mistake`] words the header and
//! [`structural_mistake_help`] the fix.

use crate::code::DUPLICATE_CHECK;

/// A structural mistake the driver recovered from the generated items an error lands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralMistake {
    /// One `check_components!` table checks `entry` (a component, with its parameters when it
    /// takes any, as the table wrote them) twice for `context`.
    DuplicateCheck { context: String, entry: String },
}

/// The header that replaces the compiler's message for a structural mistake.
pub fn plan_structural_mistake(mistake: &StructuralMistake) -> String {
    match mistake {
        StructuralMistake::DuplicateCheck { context, entry } => format!(
            "[{DUPLICATE_CHECK}] `check_components!` checks `{entry}` twice for `{context}`"
        ),
    }
}

/// The `help` naming the fix for a structural mistake.
pub fn structural_mistake_help(mistake: &StructuralMistake) -> String {
    match mistake {
        StructuralMistake::DuplicateCheck { entry, .. } => {
            format!("remove one of the two `{entry}` entries")
        }
    }
}
