//! The rustc-free model and wording for the structural mistakes in a wiring or its checks.
//!
//! Some mistakes are in the shape of what the macros were given rather than in what the wiring
//! resolves to: a check table that lists one entry twice, two tables whose check traits share a
//! name, a check entry that names no parameters for a component that takes some, a `for` loop whose
//! key does not use its variable, a `#[default_impl]` on a generic impl, or a wiring entry whose
//! generic parameters its key does not use. The compiler rejects the code the macro generated from
//! it with a bare coherence, name, or trait error about generated items (`__CheckApp`, an impl's
//! type parameter), which names neither the table nor the entry. The driver recovers the mistake
//! from the generated items and fills [`StructuralMistake`] in; [`plan_structural_mistake`] words
//! the header and [`structural_mistake_help`] the fix.

use crate::code::{
    CHECK_AT_UNIT_PARAMETERS, DUPLICATE_CHECK, DUPLICATE_CHECK_TRAIT, ENTRY_PARAMETERS_NOT_IN_KEY,
    GENERIC_DEFAULT_IMPL, LOOP_KEY_WITHOUT_VARIABLE, SHARED_DISPATCH_METHOD,
};
use crate::diagnosis::wording::quoted_list;

/// The label on the first of two check tables whose check traits share a name.
pub const FIRST_CHECK_TABLE_LABEL: &str = "the first table";

/// The label on the second of two check tables whose check traits share a name, the
/// one the compiler reports.
pub const SECOND_CHECK_TABLE_LABEL: &str = "the second table";

/// The label on the first of two `#[cgp_auto_dispatch]` methods of one name in one module.
pub const FIRST_DISPATCH_METHOD_LABEL: &str = "declared first here";

/// The label on the second of two `#[cgp_auto_dispatch]` methods of one name in one module, the one
/// the compiler reports.
pub const SECOND_DISPATCH_METHOD_LABEL: &str = "declared again here";

/// A structural mistake the driver recovered from the generated items an error lands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralMistake {
    /// One `check_components!` table checks `entry` (a component, with its parameters when it
    /// takes any, as the table wrote them) twice for `context`.
    DuplicateCheck { context: String, entry: String },
    /// A check entry for `entry` on `context`, written in `form`, names no parameters for a
    /// component that takes some, so it checks the component at the unit parameters `()`;
    /// `parameters` are those a provider implements and the context passes the check at, each as
    /// `#[check_params(…)]` writes one check's parameters (a tuple for several).
    CheckAtUnitParameters {
        context: String,
        entry: String,
        parameters: Vec<String>,
        form: CheckForm,
    },
    /// Two `#[cgp_auto_dispatch]` traits in one module declare `method`, so the macro generates its
    /// `helper` function and its `computer` provider twice.
    SharedDispatchMethod {
        method: String,
        helper: String,
        computer: String,
    },
    /// Two check tables in one module generate check traits of the same name, either both derived
    /// by one macro from contexts whose paths end in `context` or both named `check_trait` by a
    /// `#[check_trait(...)]` attribute. `first` and `second` are the macros of the tables in
    /// written order.
    DuplicateCheckTrait {
        name: CheckTraitName,
        first: CheckForm,
        second: CheckForm,
    },
    /// A `for <variable, …> in Table` loop in the wiring of `context` whose `key` does not use
    /// `variable`, the loop's key variable, so the variable is bound to nothing.
    LoopKeyWithoutVariable {
        context: String,
        variable: String,
        key: String,
    },
    /// A `#[default_impl(key in …)]` on the impl of `provider` that is generic over `parameter`,
    /// which the registration copies but nothing in it uses. `provider_at_key` is the provider with
    /// `key` in place of `parameter`, when the provider struct itself uses the parameter
    /// (`ShowWith<String>` for `ShowWith<T>`).
    GenericDefaultImpl {
        provider: String,
        key: String,
        parameter: String,
        provider_at_key: Option<String>,
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

/// The macro a check entry is written in, which decides how its parameters are written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckForm {
    /// `check_components!`, which writes them after the component (`Component: Params`).
    CheckComponents,
    /// `delegate_and_check_components!`, which writes them in a `#[check_params(…)]` attribute.
    DelegateAndCheckComponents,
}

impl CheckForm {
    /// The macro as a program calls it, for a message that names it.
    pub fn macro_name(&self) -> &'static str {
        match self {
            CheckForm::CheckComponents => "check_components!",
            CheckForm::DelegateAndCheckComponents => "delegate_and_check_components!",
        }
    }
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

/// The name two check tables gave their check traits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckTraitName {
    /// The name the macro derives from the last segment of the context's path: `__Check{context}`
    /// for `check_components!`, and `__CanUse{context}` for `delegate_and_check_components!`.
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
        StructuralMistake::CheckAtUnitParameters { context, entry, .. } => format!(
            "[{CHECK_AT_UNIT_PARAMETERS}] the check of `{entry}` on `{context}` names no parameters, so it checks the component at the unit parameters `()`"
        ),
        StructuralMistake::SharedDispatchMethod { method, .. } => format!(
            "[{SHARED_DISPATCH_METHOD}] two `#[cgp_auto_dispatch]` traits in one module declare the method `{method}`"
        ),
        StructuralMistake::DuplicateCheckTrait {
            name: CheckTraitName::Derived { context },
            second,
            ..
        } => format!(
            "[{DUPLICATE_CHECK_TRAIT}] two `{}` tables in one module check `{context}`",
            second.macro_name()
        ),
        StructuralMistake::DuplicateCheckTrait {
            name: CheckTraitName::Given { check_trait },
            first,
            second,
        } => format!(
            "[{DUPLICATE_CHECK_TRAIT}] {} in one module name their check trait `{check_trait}`",
            check_tables(first, second)
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
            ..
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
        StructuralMistake::CheckAtUnitParameters {
            entry,
            parameters,
            form,
            ..
        } => {
            let check = match (form, parameters.as_slice()) {
                (CheckForm::DelegateAndCheckComponents, _) => {
                    format!("#[check_params({})]", parameters.join(", "))
                }
                (CheckForm::CheckComponents, [single]) => format!("{entry}: {single}"),
                (CheckForm::CheckComponents, _) => {
                    format!("{entry}: [{}]", parameters.join(", "))
                }
            };
            format!("check it at the parameters its provider implements: `{check}`")
        }
        StructuralMistake::SharedDispatchMethod { method, .. } => format!(
            "rename `{method}` in one of the two traits, or declare the traits in different modules"
        ),
        StructuralMistake::DuplicateCheckTrait {
            name: CheckTraitName::Derived { .. },
            ..
        } => "merge the two tables, or name one table's check trait with `#[check_trait(...)]`"
            .to_owned(),
        StructuralMistake::DuplicateCheckTrait {
            name: CheckTraitName::Given { .. },
            first,
            second,
        } if first == second => {
            "merge the two tables, or give one of them another `#[check_trait(...)]` name"
                .to_owned()
        }
        StructuralMistake::DuplicateCheckTrait {
            name: CheckTraitName::Given { .. },
            ..
        } => "give one of the tables another `#[check_trait(...)]` name".to_owned(),
        StructuralMistake::LoopKeyWithoutVariable { variable, .. } => {
            format!("write `{variable}` in the key where the type the loop covers goes")
        }
        StructuralMistake::GenericDefaultImpl {
            provider,
            key,
            parameter,
            provider_at_key: None,
        } => format!(
            "write `{provider}`'s impl for `{key}` alone, in the concise `#[cgp_impl]` form without `{parameter}`"
        ),
        StructuralMistake::GenericDefaultImpl {
            key,
            parameter,
            provider_at_key: Some(provider_at_key),
            ..
        } => format!(
            "write the impl for `{key}` alone, as `#[cgp_impl({provider_at_key})]` with `{key}` in place of `{parameter}`"
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

/// The `note` explaining a structural mistake, for the mistakes whose header alone does not say why
/// the program fails.
pub fn structural_mistake_note(mistake: &StructuralMistake) -> Option<String> {
    match mistake {
        StructuralMistake::SharedDispatchMethod {
            helper, computer, ..
        } => Some(format!(
            "`#[cgp_auto_dispatch]` names the items it generates after the method alone, so both traits generate `{computer}` and `{helper}`"
        )),
        _ => None,
    }
}

/// The two check tables of a [`StructuralMistake::DuplicateCheckTrait`], each named by its macro:
/// "two `check_components!` tables" for one macro, and "a `check_components!` table and a
/// `delegate_and_check_components!` table", in written order, for two.
fn check_tables(first: &CheckForm, second: &CheckForm) -> String {
    if first == second {
        format!("two `{}` tables", first.macro_name())
    } else {
        format!(
            "a `{}` table and a `{}` table",
            first.macro_name(),
            second.macro_name()
        )
    }
}
