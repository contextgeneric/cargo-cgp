//! The rustc-free model and wording for an orphan-rule namespace-registration failure.
//!
//! Registering wiring into a namespace lowers to `impl<Param> Namespace<Param> for Key`. Rust's
//! orphan rule accepts a foreign-trait impl only when a local type covers its parameters, so when
//! *both* the namespace trait and the key are foreign — a downstream crate registering into an
//! upstream namespace it does not own, keyed on an upstream component it does not own either —
//! nothing is local and the compiler rejects it with `E0210` (or its sibling `E0117`). The raw
//! diagnostic is accurate but frames a CGP wiring decision as a bare coherence rule, naming the
//! machinery parameter (`__Components__` / `__Table__`) rather than the namespace and key the
//! programmer wrote.
//!
//! This model records what the driver recovers from the offending impl — the foreign namespace,
//! the key, and which construct registered it — and [`plan_orphan_conflict`] words it into the
//! `[CGP-E011]` header that replaces the raw message, with [`orphan_conflict_help`] carrying the
//! ownership-based fix. Keeping the model and wording here, in owned `String` form, is what makes
//! them unit-testable without a compiler; the driver's `resolve::orphan` module fills the model in
//! from the live `TyCtxt`.

use crate::code::ORPHAN_FOREIGN_NAMESPACE;
use crate::diagnosis::wiring::WiringKey;

/// Which construct registered the wiring the orphan rule rejected — the two differ only in the
/// idiomatic fix, since the violation (a foreign namespace trait implemented for a foreign key) is
/// identical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrphanTrigger {
    /// A `#[default_impl(... in Namespace)]` or `#[prefix(... in Namespace)]` registering a
    /// component (or per-type default) into the namespace under a component or path key.
    Register,
    /// A `cgp_namespace! { ForeignNamespace { … } }` block *without* `new`, re-opening a foreign
    /// namespace to add an entry.
    Reopen,
}

/// A recognized orphan-rule namespace registration: a foreign namespace trait implemented locally
/// for a foreign key. Carries the namespace trait name, the key in the programmer's own surface
/// form (a bare component marker or an `@`-path, never a [`Blanket`](WiringKey::Blanket)), and the
/// construct that generated the impl.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanConflict {
    pub namespace: String,
    pub key: WiringKey,
    pub trigger: OrphanTrigger,
}

impl WiringKey {
    /// The key as the object of "cannot register the foreign …" — a bare component or an `@`-path.
    fn orphan_phrase(&self) -> String {
        match self {
            WiringKey::Component(name) => format!("component `{name}`"),
            WiringKey::Path(path) => format!("path `{path}`"),
            // The orphan classifier only ever recovers a component or path key; a blanket
            // forwarding is not a registration, so this arm is unreachable in practice.
            WiringKey::Blanket(name) => format!("`{name}`"),
        }
    }
}

/// Word an [`OrphanConflict`] into the `[CGP-E011]` header that replaces the raw `E0210`/`E0117`
/// message, naming the foreign namespace and key rather than the machinery parameter. The kept
/// caret (which the emitter leaves pointing at the offending macro) says *where*; this says *what*.
pub fn plan_orphan_conflict(conflict: &OrphanConflict) -> String {
    format!(
        "[{ORPHAN_FOREIGN_NAMESPACE}] cannot register the foreign {} into the foreign namespace \
         `{}`",
        conflict.key.orphan_phrase(),
        conflict.namespace,
    )
}

/// The `help` accompanying the header — the ownership-based fix, which is the one thing the raw
/// coherence error never states. It differs by [`OrphanTrigger`]: a registration is fixed by owning
/// one end (a local key, or registering from the namespace's own crate), while a re-open is fixed by
/// inheriting the namespace into a new local one instead of extending it in place.
pub fn orphan_conflict_help(conflict: &OrphanConflict) -> String {
    match conflict.trigger {
        OrphanTrigger::Register => format!(
            "own one end of the wiring: key it on a component defined in this crate, or register \
             it from the crate that defines `{}`",
            conflict.namespace,
        ),
        OrphanTrigger::Reopen => format!(
            "to extend a foreign namespace, define a new local namespace that inherits it: \
             `cgp_namespace! {{ new MyNamespace: {} {{ … }} }}`",
            conflict.namespace,
        ),
    }
}
