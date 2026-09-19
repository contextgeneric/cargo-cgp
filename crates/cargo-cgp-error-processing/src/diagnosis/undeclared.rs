//! The rustc-free model and wording for an undeclared-trait failure.
//!
//! `#[cgp_fn]` and `#[cgp_impl]` lower a function into a blanket impl over a generated generic
//! context — `impl<__Context__> Describe for __Context__ where __Context__: GetName { … }`. The
//! body may call *other* CGP traits on `self`, but each must be declared as a dependency
//! (with `#[uses(…)]`) so it becomes a `where` bound on that generic context. When the body calls a
//! trait the `#[uses]` list omits, the compiler reports a vague `E0599` — "the method `…`
//! exists for reference `&__Context__`, but its trait bounds were not satisfied" — naming the
//! generated `__Context__` the programmer never wrote and pointing at a *transitive* missing bound
//! (a `HasField`) rather than the real fix: declaring the trait.
//!
//! This model records the trait the body used but did not declare, and
//! [`plan_undeclared_trait`] words it into the `[CGP-E012]` header that replaces the raw
//! message, with [`undeclared_trait_help`] carrying the `#[uses(…)]` fix. The driver's
//! `resolve::undeclared` module fills the model in from the live `TyCtxt`; keeping the wording here
//! in owned `String` form makes it unit-testable without a compiler.

use crate::code::UNDECLARED_TRAIT;

/// A CGP trait a `#[cgp_fn]`/`#[cgp_impl]` body calls without declaring it as a dependency.
/// `trait_name` is the trait's name (a CGP consumer trait, or a `#[cgp_fn]`/`#[blanket_trait]`
/// blanket trait) — the name to add to `#[uses(…)]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndeclaredTrait {
    pub trait_name: String,
}

/// Word an [`UndeclaredTrait`] into the `[CGP-E012]` header that replaces the raw `E0599`,
/// naming the trait rather than the generated `__Context__` and its transitive `HasField`
/// bound. The kept caret (left on the method call) says *where*; this says *what*.
pub fn plan_undeclared_trait(undeclared: &UndeclaredTrait) -> String {
    format!(
        "[{UNDECLARED_TRAIT}] the trait `{}` is used but not declared as a dependency",
        undeclared.trait_name,
    )
}

/// The `help` accompanying the header — the fix the raw error never states: declare the trait
/// with `#[uses(…)]` so it becomes a bound on the generated context and the method can be called.
pub fn undeclared_trait_help(undeclared: &UndeclaredTrait) -> String {
    format!(
        "declare it as a dependency with `#[uses({})]`",
        undeclared.trait_name,
    )
}
