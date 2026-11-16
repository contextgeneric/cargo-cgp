//! The rustc-free model and wording for an undeclared-trait failure (`[CGP-E030]`).
//!
//! See `cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md`.

use crate::code::UNDECLARED_TRAIT;
use crate::diagnosis::wording::quoted_list;

/// A CGP trait a `#[cgp_fn]`/`#[cgp_impl]` body calls without declaring it as a dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndeclaredTrait {
    /// The trait's name (a CGP consumer trait, or a `#[cgp_fn]`/`#[blanket_trait]` blanket trait),
    /// as the header states it.
    pub trait_name: String,
    /// The bound the body needs, as `#[uses(…)]` writes it: the trait with the arguments the call
    /// fixes (`CanFormatPair<(u32, u64)>`), each argument the call does not give away written as
    /// the trait's own parameter name. `None` when the tool cannot write the bound at all.
    pub bound: Option<String>,
    /// The trait's parameters the call does not give away, named in `bound` for the reader to
    /// replace.
    pub unresolved: Vec<String>,
}

/// Word an [`UndeclaredTrait`] into the `[CGP-E030]` header that replaces the raw `E0599`,
/// naming the trait rather than the generated `__Context__` and its transitive `HasField`
/// bound. The kept caret (left on the method call) says *where*; this says *what*.
pub fn plan_undeclared_trait(undeclared: &UndeclaredTrait) -> String {
    format!(
        "[{UNDECLARED_TRAIT}] the trait `{}` is used but not declared as a dependency",
        undeclared.trait_name,
    )
}

/// The `help` accompanying the header, the fix the raw error never states: declare the trait
/// with `#[uses(…)]` so it becomes a bound on the generated context and the call can resolve.
/// `None` when the bound cannot be written. A parameter the call does not give away stays named,
/// and the help says to replace it.
pub fn undeclared_trait_help(undeclared: &UndeclaredTrait) -> Option<String> {
    let bound = undeclared.bound.as_ref()?;
    let declare = format!("declare it as a dependency with `#[uses({bound})]`");
    Some(match undeclared.unresolved.len() {
        0 => declare,
        1 => format!(
            "{declare}, with the type the call uses in place of {}",
            quoted_list(&undeclared.unresolved),
        ),
        _ => format!(
            "{declare}, with the types the call uses in place of {}",
            quoted_list(&undeclared.unresolved),
        ),
    })
}
