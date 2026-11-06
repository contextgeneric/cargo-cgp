//! The rustc-free model and wording for an abstract type used but not imported.
//!
//! A CGP definition names an abstract type such as the context's `Error` through
//! `#[use_type(HasErrorType.Error)]`, which rewrites each bare `Error` to
//! `<Self as HasErrorType>::Error` and adds the `HasErrorType` bound. Written without the import,
//! `Self::Error` (or `__Context__::Error`, as `#[cgp_impl]` rewrites it) names an associated type
//! of a context that carries no bound declaring one, and the compiler reports `E0220`, "associated
//! type `Error` not found for `__Context__`", naming the generated context and no fix.
//!
//! The driver recovers the abstract type and the trait that declares it, and
//! [`plan_unimported_abstract_type`] words the `[CGP-E035]` header that replaces the message, with
//! [`unimported_abstract_type_help`] carrying the `#[use_type(…)]` import.

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
