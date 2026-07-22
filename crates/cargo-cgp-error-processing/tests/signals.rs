//! The text signals — the stable rustc phrasings the emitter routes decisions on — over plain
//! strings. Each predicate is a *signal*, not a classification, so these pin the exact wordings it
//! matches and, as importantly, the near-misses it must not.

use cargo_cgp_error_processing::{
    is_method_bounds_text, is_method_probe_advice_text, is_question_mark_cascade_text,
    mentions_orphan_param_text, mentions_wiring_text,
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
