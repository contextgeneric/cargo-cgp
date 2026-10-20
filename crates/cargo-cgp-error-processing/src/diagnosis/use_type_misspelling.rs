//! The rustc-free model and wording for a `#[use_type]` import that misspells its associated type.
//!
//! `#[use_type(HasScalarType.Scalr)]` rewrites each bare `Scalr` to `<Self as HasScalarType>::Scalr`,
//! which names an associated type the trait does not declare, so the compiler reports `E0576` with
//! its caret on the alias the program wrote. The compiler's own help respells that alias, which
//! leaves the import's name in place and fails on the respelled one. The driver confirms that the
//! path came from the import, and [`use_type_misspelling_help`] names the rename that compiles.

/// An associated type a `#[use_type]` import names that its trait does not declare, with the one
/// the trait declares that the name is a slip of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseTypeMisspelling {
    /// The name the import gives, e.g. `Scalr`.
    pub type_name: String,
    /// The trait the import names, e.g. `HasScalarType`.
    pub trait_name: String,
    /// The associated type of the trait the name is a slip of, e.g. `Scalar`.
    pub similar: String,
    /// Whether the definition uses the misspelled name itself as the alias, so each use is renamed
    /// with the import; an `as` alias keeps its own name.
    pub alias_is_name: bool,
}

/// The `help` naming the rename in the import, and in each use of the alias when the alias is the
/// misspelled name.
pub fn use_type_misspelling_help(misspelling: &UseTypeMisspelling) -> String {
    let UseTypeMisspelling {
        type_name,
        trait_name,
        similar,
        alias_is_name,
    } = misspelling;
    let mut help = format!(
        "rename `{type_name}` to `{similar}` in the `#[use_type]` import from `{trait_name}`"
    );
    if *alias_is_name {
        help.push_str(&format!(" and in each use of `{type_name}`"));
    }
    help
}
