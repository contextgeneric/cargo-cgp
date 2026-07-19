//! Text signals — the stable rustc phrasings the emitter routes decisions on.
//!
//! The emitter must decide, from a diagnostic's rendered messages alone, whether to attempt a
//! transform at all; these predicates centralize the rustc wordings those decisions key on, so
//! each is documented and unit-tested in one place. They are *signals*, not classifications: a
//! positive match only makes a diagnostic a candidate, and the typed resolver (or the span checks
//! around a cascade) still verifies the real condition before anything is rewritten or dropped.

/// Whether a message mentions a CGP wiring trait — the cheap pre-filter that makes a diagnostic a
/// typed-resolution candidate even when its rustc code alone would not. `HasField` catches a
/// use-site failure (a consumer-method `E0599`), whose text names the missing leaf but not
/// `CanUseComponent`/`IsProviderFor`.
pub fn mentions_wiring_text(text: &str) -> bool {
    text.contains("CanUseComponent") || text.contains("IsProviderFor") || text.contains("HasField")
}

/// Whether a main message is the method-bounds `E0599` shape — "the method `…` exists … but its
/// trait bounds were not satisfied" — the one `E0599` form the typed resolver may run on. A
/// *resolution*-class `E0599` (`no variant named …`) is emitted mid-`predicates_of`, where running
/// the solver re-enters the diagnostic context and aborts the compiler, so the distinction is
/// load-bearing (see `docs/implementation/rustc-diagnostic-internals.md`).
pub fn is_method_bounds_text(text: &str) -> bool {
    text.contains("trait bounds were not satisfied")
}

/// Whether a message is part of rustc's method-probe advice on a CGP consumer-method failure —
/// the "this is an associated function, not a method" caret label and the "found the following
/// associated functions …" note (with its "the candidate is defined in …" follow-up). Both are
/// artifacts of CGP's `self`-less provider methods: the probe sees the provider trait's
/// associated fn and concludes the *call syntax* is wrong, when the real fault is an unmet
/// wiring bound the same diagnostic already names. The emitter drops messages matching this on a
/// method-bounds `E0599` that mentions a CGP wiring trait, so the misleading advice never
/// outranks the real cause.
pub fn is_method_probe_advice_text(text: &str) -> bool {
    text.contains("this is an associated function, not a method")
        || text.starts_with("found the following associated functions")
        || text.starts_with("the candidate is defined in")
        || text.starts_with("the candidates are defined in")
}

/// Whether a main message is a `?`-operator error — the `Try`/`FromResidual` shape rustc emits
/// when `expr?` is applied to a value whose type it could not resolve because an earlier trait
/// bound on that same expression failed. Both `Try` shapes share rustc's stable "the `?` operator
/// can only be …" wording. On its own this is not enough to suppress — a genuine `?` misuse reads
/// the same — so the emitter pairs it with a span check: the cascade is dropped only when it sits
/// on an expression where a CGP wiring failure was already reported.
pub fn is_question_mark_cascade_text(text: &str) -> bool {
    text.contains("`?` operator")
}
