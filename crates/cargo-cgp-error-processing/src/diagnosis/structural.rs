//! The rustc-free model and wording for the structural mistakes in a wiring or its checks.
//!
//! Some mistakes are in the shape of what the macros were given rather than in what the wiring
//! resolves to: a check table that lists one entry twice, two tables whose check traits share a
//! name, a `for` loop whose key does not use its variable, a `#[default_impl]` on a generic impl,
//! or a wiring entry whose generic parameters its key does not use. The compiler rejects the code
//! the macro generated from it with a bare coherence or name error about generated items
//! (`__CheckApp`, an impl's type parameter), which names neither the table nor the entry. The driver recovers the mistake from the generated
//! items and fills [`StructuralMistake`] in; [`plan_structural_mistake`] words the header and
//! [`structural_mistake_help`] the fix.

use crate::code::{
    DUPLICATE_CHECK, DUPLICATE_CHECK_TRAIT, ENTRY_PARAMETERS_NOT_IN_KEY, GENERIC_DEFAULT_IMPL,
    LOOP_KEY_WITHOUT_VARIABLE,
};
use crate::diagnosis::wording::quoted_list;

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
    /// A `for <variable, …> in Table` loop in the wiring of `context` whose `key` does not use
    /// `variable`, the loop's key variable, so the variable is bound to nothing.
    LoopKeyWithoutVariable {
        context: String,
        variable: String,
        key: String,
    },
    /// A `#[default_impl(key in …)]` on the impl of `provider` that is generic over `parameter`,
    /// which the registration copies but nothing in it uses.
    GenericDefaultImpl {
        provider: String,
        key: String,
        parameter: String,
    },
    /// An entry for `key` in `table` binds generic `parameters` (in written order) that its key
    /// does not use, so the impls it lowers to cannot bind them; `used` says what in the entry
    /// uses them.
    EntryParametersNotInKey {
        table: EntryTable,
        key: String,
        parameters: Vec<String>,
        used: ParameterUse,
    },
}

/// The wiring table an entry belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryTable {
    /// The wiring of a context, written with `delegate_components!` or
    /// `delegate_and_check_components!`.
    Context(String),
    /// A namespace defined with `cgp_namespace!`.
    Namespace(String),
}

/// What in an entry uses the generic parameters its key does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterUse {
    /// The provider the entry maps its key to, as written (`GreetWith<T>`).
    Provider(String),
    /// Another part of the entry's value, such as the table a `->` entry forwards to.
    Value,
    /// Nothing: the entry declares the parameters and never uses them.
    Unused,
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
        StructuralMistake::LoopKeyWithoutVariable {
            context,
            variable,
            key,
        } => format!(
            "[{LOOP_KEY_WITHOUT_VARIABLE}] the `for` loop in the wiring of `{context}` binds `{variable}`, but its key `{key}` does not use it"
        ),
        StructuralMistake::GenericDefaultImpl {
            provider,
            key,
            parameter,
        } => format!(
            "[{GENERIC_DEFAULT_IMPL}] `#[default_impl]` cannot register `{provider}` for `{key}`, because the provider's impl is generic over `{parameter}`"
        ),
        StructuralMistake::EntryParametersNotInKey {
            table,
            key,
            parameters,
            ..
        } => {
            let table = match table {
                EntryTable::Context(context) => format!("the wiring of `{context}`"),
                EntryTable::Namespace(namespace) => format!("the namespace `{namespace}`"),
            };
            let pronoun = if parameters.len() == 1 { "it" } else { "them" };
            format!(
                "[{ENTRY_PARAMETERS_NOT_IN_KEY}] the entry for `{key}` in {table} binds {}, but its key does not use {pronoun}",
                quoted_list(parameters),
            )
        }
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
        StructuralMistake::LoopKeyWithoutVariable { variable, .. } => {
            format!("write `{variable}` in the key where the type the loop covers goes")
        }
        StructuralMistake::GenericDefaultImpl {
            provider,
            key,
            parameter,
        } => format!(
            "write `{provider}`'s impl for `{key}` alone, in the concise `#[cgp_impl]` form without `{parameter}`"
        ),
        StructuralMistake::EntryParametersNotInKey {
            parameters, used, ..
        } => {
            let names = quoted_list(parameters);
            let remove = format!("remove {names} from the entry's generic parameters");
            let (types, pronoun) = if parameters.len() == 1 {
                ("a concrete type", "it")
            } else {
                ("concrete types", "them")
            };
            match used {
                ParameterUse::Provider(provider) => {
                    format!("write {types} in place of {names} in `{provider}`, and {remove}")
                }
                ParameterUse::Value => {
                    format!(
                        "write {types} in place of {names} where the entry uses {pronoun}, and {remove}"
                    )
                }
                ParameterUse::Unused => remove,
            }
        }
    }
}
