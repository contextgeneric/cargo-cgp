//! The rustc-free model and wording for an undeclared-trait failure.
//!
//! `#[cgp_fn]` and `#[cgp_impl]` lower a function into a blanket impl over a generated generic
//! context, `impl<__Context__> Describe for __Context__ where __Context__: GetName { … }`. The
//! body may call *other* CGP traits on `self`, or as associated functions on `Self`, but each must
//! be declared as a dependency (with `#[uses(…)]`) so it becomes a `where` bound on that generic
//! context. When the body calls a trait the `#[uses]` list omits, the compiler reports a vague
//! `E0599` ("the method `…` exists for reference `&__Context__`, but its trait bounds were not
//! satisfied", or "no associated function … found for type parameter `__Context__`"). It names the
//! generated `__Context__` the programmer never wrote and points at a transitive bound or a
//! suggestion spliced into the macro attribute, rather than at the real fix: declaring the trait.
//!
//! This model records the trait the body used but did not declare, and
//! [`plan_undeclared_trait`] words it into the `[CGP-E030]` header that replaces the raw
//! message, with [`undeclared_trait_help`] carrying the `#[uses(…)]` fix. The driver's
//! `resolve::undeclared` module fills the model in from the live `TyCtxt`; keeping the wording here
//! in owned `String` form makes it unit-testable without a compiler.

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
