//! Text signals: the stable rustc phrasings the emitter routes decisions on.
//!
//! The emitter must decide, from a diagnostic's rendered messages alone, whether to attempt a
//! transform at all; these predicates centralize the rustc wordings those decisions key on, so
//! each is documented and unit-tested in one place. The readers that take a name out of a message
//! and the writers of the footer lines the emitter rebuilds live here too, beside the wording they
//! share. They are *signals*, not classifications: a
//! positive match only makes a diagnostic a candidate, and the typed resolver (or the span checks
//! around a cascade) still verifies the real condition before anything is rewritten or dropped.

/// Whether a message mentions a CGP wiring trait: the cheap pre-filter that makes a diagnostic a
/// typed-resolution candidate even when its rustc code alone would not. `HasField` catches a
/// use-site failure (a consumer-method `E0599`), whose text names the missing leaf but not
/// `CanUseComponent`/`IsProviderFor`.
pub fn mentions_wiring_text(text: &str) -> bool {
    text.contains("CanUseComponent") || text.contains("IsProviderFor") || text.contains("HasField")
}

/// Whether a main message is the method-bounds `E0599` shape, "the method `…` exists … but its
/// trait bounds were not satisfied". It is the one `E0599` form the typed resolver may run on. A
/// *resolution*-class `E0599` (`no variant named …`) is emitted mid-`clauses_of`, where running
/// the solver re-enters the diagnostic context and aborts the compiler, so the distinction is
/// load-bearing (see `cgp-knowledge-base/cargo-cgp/implementation/rustc-diagnostic-internals.md`).
pub fn is_method_bounds_text(text: &str) -> bool {
    text.contains("trait bounds were not satisfied")
}

/// Whether a message is rustc's "item on an unbounded type parameter" `E0599` help: "items from
/// traits can only be used if the type parameter is bounded by the trait". This is the failure a
/// higher-order provider hits when it calls an inner provider it never imported with
/// `#[use_provider]`: the inner parameter carries no provider-trait bound, so the
/// associated-function call cannot resolve. The `E0599`'s main message is a Fluent (non-`Str`)
/// message, so this signal keys on the help, which *is* a plain string. It is distinctive to this
/// shape, reported during typeck of the calling body, where the queries the detector forces are
/// already cached, and absent from the resolution-class `E0599` emitted mid-`clauses_of` (where
/// running those queries would re-enter the diagnostic context and abort the compiler).
pub fn is_unbounded_type_param_item_text(text: &str) -> bool {
    text.contains("the type parameter is bounded by the trait")
}

/// Whether a message is part of rustc's method-probe advice on a CGP consumer-method failure: the
/// "this is an associated function, not a method" caret label, the "found the following
/// associated functions …" note (with its "the candidate is defined in …" follow-up), and the
/// "use associated function syntax instead" suggestion. All are artifacts of CGP's `self`-less
/// provider methods: the probe sees the provider trait's associated fn and concludes the *call
/// syntax* is wrong, when the real fault is an unmet wiring bound the same diagnostic already
/// names. The emitter drops messages matching this on a method-bounds `E0599` that mentions a CGP
/// wiring trait, so the misleading advice never outranks the real cause, and keeps rustc's other
/// suggestions.
pub fn is_method_probe_advice_text(text: &str) -> bool {
    text.contains("this is an associated function, not a method")
        || text.starts_with("found the following associated functions")
        || text.starts_with("the candidate is defined in")
        || text.starts_with("the candidates are defined in")
        || text.starts_with("use associated function syntax instead")
}

/// Whether a message is rustc's "you might be missing a type parameter" suggestion, which an
/// unresolved name gets when it could be a generic parameter of the item around it. A wiring
/// entry's key or value is the argument of a generated impl, which has no generics to add the name
/// to, so on a type written in an entry's `@`-path the suggestion is wrong.
pub fn is_missing_type_parameter_help_text(text: &str) -> bool {
    text.starts_with("you might be missing a type parameter")
}

/// Whether a main message is the orphan-rule `E0210` naming a CGP machinery type parameter: "type
/// parameter `__Components__` must be used as an argument to some local type". The
/// double-underscore parameter (`__Components__` from a `#[default_impl]`/`#[prefix]` registration,
/// `__Table__` from a `cgp_namespace!` re-open) is a reserved identifier the CGP macros emit, so
/// its presence in this coherence error is the cheap pre-filter that makes the diagnostic a
/// namespace-orphan candidate. It is a *signal*, not a classification: the typed classifier still
/// confirms a foreign namespace trait is implemented for a foreign key before anything is
/// rewritten.
pub fn mentions_orphan_param_text(text: &str) -> bool {
    text.contains("must be used as an argument to some local type") && text.contains("`__")
}

/// Whether a main message is a `?`-operator error: the `Try`/`FromResidual` shape rustc emits
/// when `expr?` is applied to a value whose type it could not resolve because an earlier trait
/// bound on that same expression failed. Both `Try` shapes share rustc's stable "the `?` operator
/// can only be …" wording. On its own this is not enough to suppress: a genuine `?` misuse reads
/// the same, so the emitter pairs it with a span check: the cascade is dropped only when it sits
/// on an expression where a CGP wiring failure was already reported.
pub fn is_question_mark_cascade_text(text: &str) -> bool {
    text.contains("`?` operator")
}

/// Whether a `help` is rustc's advice to raise the recursion limit, which an `E0275` overflow
/// carries. A wiring lookup that never resolves is not fixed by a higher limit, so the emitter
/// drops it from a rewritten wiring overflow.
pub fn is_recursion_limit_help_text(text: &str) -> bool {
    text.contains("consider increasing the recursion limit")
}

/// The lead of rustc's footer line that lists the explainable codes.
const EXPLAIN_LIST_LEAD: &str = "Some errors have detailed explanations:";

/// The lead of rustc's footer line that points at `rustc --explain` for a code.
const EXPLAIN_POINTER_LEAD: &str = "For more information about";

/// Whether a message is one of rustc's trailing "detailed explanations" footer lines: the
/// `Some errors have detailed explanations: E0277, E0599.` list or the
/// `For more information about …, try `rustc --explain E0277`.` pointer that `print_error_count`
/// emits last. The emitter rebuilds these from the errors that actually survived its suppressions
/// and merges, so it has to recognize them first.
pub fn is_explain_footer_text(text: &str) -> bool {
    is_explain_list_text(text) || text.starts_with(EXPLAIN_POINTER_LEAD)
}

/// Whether a message is the list form of the footer, `Some errors have detailed explanations: …`.
pub fn is_explain_list_text(text: &str) -> bool {
    text.starts_with(EXPLAIN_LIST_LEAD)
}

/// The list footer line for `codes`, as rustc writes it:
/// `Some errors have detailed explanations: E0277, E0599.`
pub fn explain_list_text(codes: &[String]) -> String {
    format!("{EXPLAIN_LIST_LEAD} {}.", codes.join(", "))
}

/// The pointer footer line for `code`, as rustc writes it: about "this error" when `code` is the
/// only explainable one, and about "an error" otherwise.
pub fn explain_pointer_text(code: &str, only: bool) -> String {
    let subject = if only { "this error" } else { "an error" };
    format!("{EXPLAIN_POINTER_LEAD} {subject}, try `rustc --explain {code}`.")
}

/// The `rustc --explain` codes an [`is_explain_footer_text`] line names: every code of the list
/// form, or the single code of the pointer form.
///
/// A footer line always names at least one code, so an empty result means the *parse* failed (a
/// rewording upstream), not that the line names nothing. The caller keys on that to leave the
/// footer alone rather than rebuild it from nothing, since rebuilding would silently delete output.
pub fn explain_footer_codes(text: &str) -> Vec<String> {
    if let Some(list) = text.strip_prefix(EXPLAIN_LIST_LEAD) {
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

/// Whether `path`, a macro invocation's path as the compiler records it in an expansion
/// (`check_components`, or `cgp::prelude::check_components` for a call written by its path), names
/// the macro `name`: its last segment is `name` exactly. A crate's own macro whose name only ends
/// the same way, such as `table_check_components`, is a different macro.
pub fn invokes_macro(path: &str, name: &str) -> bool {
    path.rsplit("::").next() == Some(name)
}

/// The text after the lead of an `E0275`'s ``overflow evaluating the requirement `…` `` message:
/// the requirement and its closing backtick, or `None` for any other message. Each caller reads the
/// requirement off the rest itself.
pub fn overflow_requirement_rest(text: &str) -> Option<&str> {
    text.strip_prefix("overflow evaluating the requirement `")
}

/// The name an `E0425` ``cannot find type `Name` in this scope`` message names, or `None` for any
/// other message.
pub fn unresolved_type_name(text: &str) -> Option<&str> {
    text.strip_prefix("cannot find type `")?
        .strip_suffix("` in this scope")
}

/// The associated type and the trait an `E0576`
/// ``cannot find associated type `Name` in trait `Trait` `` message names, or `None` for any other
/// message.
pub fn unresolved_assoc_type(text: &str) -> Option<(&str, &str)> {
    text.strip_prefix("cannot find associated type `")?
        .strip_suffix('`')?
        .split_once("` in trait `")
}
