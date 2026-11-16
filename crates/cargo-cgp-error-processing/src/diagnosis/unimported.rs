//! The rustc-free model and wording for an abstract type used but not imported (`[CGP-E035]`).
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md.

use crate::code::ABSTRACT_TYPE_NOT_IMPORTED;

/// An abstract type a CGP definition uses without importing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnimportedAbstractType {
    /// The associated type's name, as the definition uses it (`Error`).
    pub type_name: String,
    /// The abstract-type trait that declares it (`HasErrorType`).
    pub trait_name: String,
}

/// The `[CGP-E035]` header naming the abstract type the definition uses but does not import.
pub fn plan_unimported_abstract_type(unimported: &UnimportedAbstractType) -> String {
    format!(
        "[{ABSTRACT_TYPE_NOT_IMPORTED}] the abstract type `{}` is used but not imported",
        unimported.type_name,
    )
}

/// The `help` naming the `#[use_type(…)]` that imports the abstract type.
pub fn unimported_abstract_type_help(unimported: &UnimportedAbstractType) -> String {
    format!(
        "import it with `#[use_type({}.{})]`",
        unimported.trait_name, unimported.type_name,
    )
}
