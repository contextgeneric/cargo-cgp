//! The text signals — the stable rustc phrasings the emitter routes decisions on — over plain
//! strings. Each predicate is a *signal*, not a classification, so these pin the exact wordings it
//! matches and, as importantly, the near-misses it must not.

use cargo_cgp_error_processing::{
    explain_footer_codes, is_explain_footer_text, is_method_bounds_text,
    is_method_probe_advice_text, is_question_mark_cascade_text, mentions_orphan_param_text,
    mentions_wiring_text,
};

#[test]
fn wiring_signal_matches_each_wiring_trait_and_nothing_else() {
    assert!(mentions_wiring_text(
        "the trait bound `App: CanUseComponent<GreeterComponent>` is not satisfied"
    ));
    assert!(mentions_wiring_text(
        "required for `GreetHello` to implement `IsProviderFor<GreeterComponent, App>`"
    ));
    // A use-site `E0599` names the missing `HasField` leaf but no `CanUseComponent`/`IsProviderFor`,
    // so `HasField` is what makes it a candidate.
    assert!(mentions_wiring_text(
        "doesn't satisfy `_: HasField<Symbol!(\"name\")>`"
    ));
    assert!(!mentions_wiring_text("mismatched types"));
}

#[test]
fn method_bounds_signal_tells_the_two_e0599_shapes_apart() {
    assert!(is_method_bounds_text(
        "the method `greet` exists for struct `Person`, but its trait bounds were not satisfied"
    ));
    // A resolution-class `E0599` must not match: running the resolver on it re-enters the
    // diagnostic context and aborts the compiler.
    assert!(!is_method_bounds_text(
        "no variant named `Blue` found for enum `Color`"
    ));
}

#[test]
fn method_probe_advice_signal_matches_each_artifact() {
    assert!(is_method_probe_advice_text(
        "this is an associated function, not a method"
    ));
    assert!(is_method_probe_advice_text(
        "found the following associated functions; to be used as methods, functions must have a `self` parameter"
    ));
    assert!(is_method_probe_advice_text(
        "the candidate is defined in the trait `PairFormatter`"
    ));
    assert!(is_method_probe_advice_text(
        "the candidates are defined in the trait `PairFormatter`"
    ));
    // The real unmet bound the same diagnostic carries is not probe advice — it must survive.
    assert!(!is_method_probe_advice_text(
        "trait bound `App: HasField<Symbol!(\"separator\")>` was not satisfied"
    ));
}

#[test]
fn orphan_signal_needs_both_the_phrase_and_a_reserved_marker() {
    // The CGP macros emit `__Components__`/`__Table__`, so the reserved marker beside the coherence
    // phrasing is what identifies a namespace-orphan candidate.
    assert!(mentions_orphan_param_text(
        "type parameter `__Components__` must be used as an argument to some local type"
    ));
    // A genuine non-CGP orphan error carries the phrase but no reserved marker — not a candidate.
    assert!(!mentions_orphan_param_text(
        "type parameter `T` must be used as an argument to some local type"
    ));
    // The reserved marker alone, in an unrelated message, is not the orphan shape either.
    assert!(!mentions_orphan_param_text(
        "cannot find type `__Table__` in this scope"
    ));
}

#[test]
fn question_mark_signal_matches_rustc_try_wording() {
    assert!(is_question_mark_cascade_text(
        "the `?` operator can only be applied to values that implement `Try`"
    ));
    assert!(!is_question_mark_cascade_text("mismatched types"));
}

#[test]
fn recognizes_the_explain_footer_lines() {
    assert!(is_explain_footer_text(
        "Some errors have detailed explanations: E0277, E0599."
    ));
    assert!(is_explain_footer_text(
        "For more information about this error, try `rustc --explain E0277`."
    ));
    assert!(is_explain_footer_text(
        "For more information about an error, try `rustc --explain E0271`."
    ));
    // A note that merely mentions an error is not the footer.
    assert!(!is_explain_footer_text(
        "note: required for `App` to implement `CanGreet`"
    ));
}

#[test]
fn reads_the_codes_a_footer_line_names() {
    assert_eq!(
        explain_footer_codes("Some errors have detailed explanations: E0277, E0599."),
        vec!["E0277", "E0599"]
    );
    assert_eq!(
        explain_footer_codes("For more information about this error, try `rustc --explain E0277`."),
        vec!["E0277"]
    );
    assert_eq!(
        explain_footer_codes("Some errors have detailed explanations: E0271."),
        vec!["E0271"]
    );
}

/// The guard the rebuild keys on: a footer always names a code, so parsing none means the wording
/// moved. The emitter leaves such a footer alone rather than rebuilding it out of existence.
#[test]
fn yields_no_codes_when_the_wording_moved() {
    assert!(explain_footer_codes("Some errors have detailed explanations:").is_empty());
    assert!(
        explain_footer_codes("For more information about this error, see the manual.").is_empty()
    );
}
