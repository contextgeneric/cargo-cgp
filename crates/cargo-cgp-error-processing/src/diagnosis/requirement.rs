//! The rustc-free model and wording for a component requirement a provider does not declare.
//!
//! A component's `#[extend(HasName)]` or `#[use_type(HasErrorType.Error)]` requires the trait of
//! every context, so the provider trait carries the bound in its `where` clause, and a provider
//! written with `#[cgp_impl]` must declare it too, with `#[uses(…)]` or `#[use_type(…)]`. A provider
//! that does not fails with `E0277` on the generated `__Context__`, "required by a bound in
//! `Greeter`", naming neither the provider nor the declaration it lacks.
//!
//! The driver recovers the provider trait, the provider, and each requirement the provider does not
//! declare, and [`plan_undeclared_requirement`] words the `[CGP-E036]` header, with
//! [`undeclared_requirement_helps`] carrying the declarations.

use crate::code::UNDECLARED_REQUIREMENT;
use crate::diagnosis::wording::quoted_list;

/// The requirements a provider trait places on the context that a provider does not declare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndeclaredRequirement {
    /// The provider trait whose `where` clause requires them (`Greeter`).
    pub provider_trait: String,
    /// The provider that does not declare them (`GreetHello`).
    pub provider: String,
    /// Each requirement, in the order the component declares it.
    pub requirements: Vec<Requirement>,
}

/// One requirement a provider does not declare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// The required trait, with its arguments (`HasName`, `TagFor<u32>`).
    pub bound: String,
    /// For an abstract-type component, the `#[use_type(…)]` spec that imports its type
    /// (`HasErrorType.Error`); `None` for any other trait, which `#[uses(…)]` declares.
    pub import: Option<String>,
}

/// The `[CGP-E036]` header naming the requirements the provider does not declare.
pub fn plan_undeclared_requirement(undeclared: &UndeclaredRequirement) -> String {
    let bounds: Vec<String> = undeclared
        .requirements
        .iter()
        .map(|requirement| requirement.bound.clone())
        .collect();
    let pronoun = if bounds.len() == 1 { "it" } else { "them" };
    format!(
        "[{UNDECLARED_REQUIREMENT}] the provider trait `{}` requires {} of the context, but the provider `{}` does not declare {pronoun}",
        undeclared.provider_trait,
        quoted_list(&bounds),
        undeclared.provider,
    )
}

/// The `help`s naming the declarations: one `#[uses(…)]` for the traits it declares, and one
/// `#[use_type(…)]` for the abstract types it imports.
pub fn undeclared_requirement_helps(undeclared: &UndeclaredRequirement) -> Vec<String> {
    let uses: Vec<&str> = undeclared
        .requirements
        .iter()
        .filter(|requirement| requirement.import.is_none())
        .map(|requirement| requirement.bound.as_str())
        .collect();
    let imports: Vec<&str> = undeclared
        .requirements
        .iter()
        .filter_map(|requirement| requirement.import.as_deref())
        .collect();
    let mut helps = Vec::new();
    match uses.len() {
        0 => {}
        1 => helps.push(format!(
            "declare it as a dependency with `#[uses({})]`",
            uses[0]
        )),
        _ => helps.push(format!(
            "declare them as dependencies with `#[uses({})]`",
            uses.join(", ")
        )),
    }
    match imports.len() {
        0 => {}
        1 => helps.push(format!("import it with `#[use_type({})]`", imports[0])),
        _ => helps.push(format!(
            "import them with `#[use_type({})]`",
            imports.join(", ")
        )),
    }
    helps
}
