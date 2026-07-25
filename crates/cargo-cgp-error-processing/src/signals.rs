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

/// Whether a message is rustc's "item on an unbounded type parameter" `E0599` help — "items from
/// traits can only be used if the type parameter is bounded by the trait". This is the failure a
/// higher-order provider hits when it calls an inner provider it never imported with
/// `#[use_provider]`: the inner parameter carries no provider-trait bound, so the
/// associated-function call cannot resolve. The `E0599`'s main message is a Fluent (non-`Str`)
/// message, so this signal keys on the help, which *is* a plain string. It is distinctive to this
/// shape — reported during typeck of the calling body, where the queries the detector forces are
/// already cached — and absent from the resolution-class `E0599` emitted mid-`predicates_of` (where
/// running those queries would re-enter the diagnostic context and abort the compiler).
pub fn is_unbounded_type_param_item_text(text: &str) -> bool {
    text.contains("the type parameter is bounded by the trait")
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

/// Whether a main message is the orphan-rule `E0210` naming a CGP machinery type parameter —
/// "type parameter `__Components__` must be used as an argument to some local type". The
/// double-underscore parameter (`__Components__` from a `#[default_impl]`/`#[prefix]` registration,
/// `__Table__` from a `cgp_namespace!` re-open) is a reserved identifier the CGP macros emit, so its
/// presence in this coherence error is the cheap pre-filter that makes the diagnostic a
/// namespace-orphan candidate. It is a *signal*, not a classification: the typed classifier still
/// confirms a foreign namespace trait is implemented for a foreign key before anything is rewritten.
pub fn mentions_orphan_param_text(text: &str) -> bool {
    text.contains("must be used as an argument to some local type") && text.contains("`__")
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

/// Whether a message is one of rustc's trailing "detailed explanations" footer lines — the
/// `Some errors have detailed explanations: E0277, E0599.` list or the
/// `For more information about …, try `rustc --explain E0277`.` pointer that `print_error_count`
/// emits last. The emitter rebuilds these from the errors that actually survived its suppressions
/// and merges, so it has to recognize them first.
pub fn is_explain_footer_text(text: &str) -> bool {
    text.starts_with("Some errors have detailed explanations:")
        || text.starts_with("For more information about")
}

/// The `rustc --explain` codes an [`is_explain_footer_text`] line names: every code of the list
/// form, or the single code of the pointer form.
///
/// A footer line always names at least one code, so an empty result means the *parse* failed — a
/// rewording upstream — not that the line names nothing. The caller keys on that to leave the footer
/// alone rather than rebuild it from nothing, since rebuilding would silently delete output.
pub fn explain_footer_codes(text: &str) -> Vec<String> {
    if let Some(list) = text.strip_prefix("Some errors have detailed explanations:") {
        return list
            .trim()
            .trim_end_matches('.')
            .split(',')
            .map(|code| code.trim().to_owned())
            .filter(|code| !code.is_empty())
            .collect();
    }
    text.split_once("--explain ")
        .map(|(_, rest)| {
            rest.trim_start_matches(['`', ' '])
                .trim_end_matches(['.', '`', ' '])
                .to_owned()
        })
        .filter(|code| !code.is_empty())
        .into_iter()
        .collect()
}
